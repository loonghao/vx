//! VX CLI - Command Line Interface for VX Tool Manager

use anyhow::{Context, Result};
use clap::Parser;
use vx_core::WithDependency;
use vx_ecosystem_pm::{EcosystemInstaller, InstallOptions, get_installer};
use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_paths::shims;
use vx_resolver::RuntimeRequest;
use vx_runtime::ProviderRegistry;
use vx_shim::{PackageRequest, ShimExecutor};

pub mod cli;
pub mod commands;
pub mod config;
pub mod error_handler;
pub mod output;
pub mod registry;
pub mod suggestions;
pub mod system_tools;
pub mod tracing_setup;
pub mod ui;

#[cfg(test)]
pub mod test_utils;

// Re-export for convenience
pub use cli::{Cli, OutputFormat};
pub use commands::{CommandContext, CommandHandler, GlobalOptions};
pub use output::{CommandOutput, OutputRenderer};
pub use registry::{ProviderRegistryExt, create_context, create_registry};
pub use tracing_setup::setup_tracing;

/// Main entry point for the VX CLI application
/// This function sets up the provider registry and runs the CLI
pub async fn main() -> anyhow::Result<()> {
    // Parse CLI first to check for --debug flag
    let cli = Cli::parse();

    // Build command string from raw args for metrics
    let command_str = std::env::args().collect::<Vec<_>>().join(" ");

    // Initialize unified tracing + OpenTelemetry metrics system.
    // The guard writes metrics to ~/.vx/metrics/ on drop.
    let _metrics_guard = vx_metrics::init(vx_metrics::MetricsConfig {
        debug: cli.debug,
        verbose: cli.verbose,
        command: command_str,
        ..Default::default()
    });

    // Set UI verbose mode based on CLI flags
    if cli.verbose || cli.debug {
        ui::UI::set_verbose(true);
    }

    // Normalize compact + filter-level CLI flags to env vars so that the executor
    // (which reads VX_OUTPUT / VX_FILTER_LEVEL) picks them up without needing an
    // extra parameter chain.  We only set them when not already set by the caller.
    if cli.compact && std::env::var("VX_OUTPUT").is_err() {
        // Safety: called before any threads are spawned by this process.
        #[allow(clippy::disallowed_methods)]
        unsafe {
            std::env::set_var("VX_OUTPUT", "compact");
        }
    }
    {
        use crate::cli::FilterLevelArg;
        let level_str = match cli.filter_level {
            FilterLevelArg::Light => Some("light"),
            FilterLevelArg::Aggressive => Some("aggressive"),
            FilterLevelArg::Normal => None, // Normal is the default; no need to set
        };
        if let Some(level) = level_str
            && std::env::var("VX_FILTER_LEVEL").is_err()
        {
            #[allow(clippy::disallowed_methods)]
            unsafe {
                std::env::set_var("VX_FILTER_LEVEL", level);
            }
        }
    }

    // Fast-path for lightweight commands that do not require provider registry
    // or runtime context initialization. This significantly reduces fixed startup
    // overhead for config/script read-only operations used in benchmarks.
    if let Some(result) = try_execute_lightweight_command(&cli).await {
        if result.is_err() {
            _metrics_guard.set_exit_code(1);
        }
        return result;
    }

    // Register embedded bridge binaries (e.g., MSBuild.exe on Windows)
    // This must happen before any provider tries to deploy bridges.
    registry::register_embedded_bridges();

    // Create provider registry with all available providers
    let registry = create_registry();

    // Create global options from CLI
    let options = GlobalOptions::from(&cli);

    // Create runtime context (apply global cache mode)
    let context = create_context()?.with_cache_mode(options.cache_mode);

    // Create command context
    let cmd_ctx = CommandContext::new(registry, context, options);

    // Route to appropriate handler
    let result = match &cli.command {
        Some(command) => command.execute(&cmd_ctx).await,
        None => {
            // No subcommand provided, try to execute as tool
            if cli.args.is_empty() {
                // Show help if no arguments
                Cli::parse_from(["vx", "--help"]);
                Ok(())
            } else {
                // Execute tool with --with dependencies
                execute_tool(&cmd_ctx, &cli.args, &cli.with_deps).await
            }
        }
    };

    // Set exit code for metrics
    if result.is_err() {
        _metrics_guard.set_exit_code(1);
    }

    result
}

/// Execute lightweight commands without initializing provider registry/runtime context.
///
/// Returns Some(result) when command was handled via fast-path, None otherwise.
async fn try_execute_lightweight_command(cli: &Cli) -> Option<Result<()>> {
    use crate::cli::{Commands, ConfigCommand};

    match &cli.command {
        // `vx version` only prints the compiled-in version string.
        Some(Commands::Version) => Some(commands::version::handle().await),

        // `vx config show` is used in benchmark parse tests and only needs local config I/O.
        Some(Commands::Config {
            command: Some(ConfigCommand::Show) | None,
        }) => Some(commands::config::handle(GlobalOptions::from(cli).output_format).await),

        // `vx config validate` is also benchmarked and does not require runtime/provider init.
        Some(Commands::Config {
            command: Some(ConfigCommand::Validate { path, verbose }),
        }) => Some(commands::config::handle_validate(path.clone(), *verbose).await),

        // `vx config dir` only prints a path.
        Some(Commands::Config {
            command: Some(ConfigCommand::Dir),
        }) => Some(commands::config::handle_dir().await),

        // `vx run --list` benchmark path: read/print scripts from vx.toml only.
        Some(Commands::Run {
            script: _,
            list: true,
            script_help: false,
            args: _,
        }) => Some(commands::run::handle(None, true, false, &[]).await),

        // `vx metrics` reads JSON files from disk, no registry needed.
        Some(Commands::Metrics {
            last,
            json,
            html,
            clean,
        }) => Some(commands::metrics::handle(*last, *json, html.clone(), *clean).await),

        _ => None,
    }
}

/// Execute a tool with the given arguments
///
/// Supports multiple syntaxes:
///
/// 1. Runtime execution (`runtime@version`):
///    - `vx yarn@1.21.1 global add terminalizer`
///    - `vx node@20 --version`
///
/// 2. Globally installed package executables:
///    - `vx tsc` (from typescript package)
///    - `vx eslint` (from eslint package)
///
/// 3. RFC 0027 implicit package execution (`ecosystem:package[@version][::executable]`):
///    - `vx npm:typescript::tsc --version`
///    - `vx pip:httpie::http GET example.com`
///    - `vx npm@20:typescript::tsc` (with runtime version)
///
/// 4. With --with flag for additional runtime dependencies:
///    - `vx --with bun npm:opencode-ai@latest::opencode`
///    - `vx --with bun@1.1.0 node my-script.js`
async fn execute_tool(
    ctx: &CommandContext,
    args: &[String],
    with_deps_specs: &[String],
) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("No tool specified"));
    }

    let tool_spec = &args[0];
    let tool_args: Vec<String> = args[1..].to_vec();

    // Parse --with dependencies
    let with_deps = WithDependency::parse_many(with_deps_specs);

    // Check if this is an RFC 0027 package request (ecosystem:package syntax)
    // Priority:
    // 1. Check against dynamic package_prefixes from provider registry
    // 2. Fallback to built-in list in vx-shim for backwards compatibility
    let is_pkg_req = if let Some(colon_pos) = tool_spec.find(':') {
        let ecosystem_part = &tool_spec[..colon_pos];
        let ecosystem = ecosystem_part.split('@').next().unwrap_or("");

        // Check against registry's dynamic package_prefixes
        let matches_registry = {
            let reg = vx_starlark::handle::global_registry().await;
            reg.get_all_package_prefixes()
                .iter()
                .any(|p| p.eq_ignore_ascii_case(ecosystem))
        };

        // Also check built-in list for backwards compatibility
        let matches_builtin = PackageRequest::is_package_request(tool_spec);

        matches_registry || matches_builtin
    } else {
        false
    };

    tracing::debug!(
        "execute_tool: tool_spec={}, is_package_request={}, with_deps={:?}",
        tool_spec,
        is_pkg_req,
        with_deps
    );

    if is_pkg_req {
        // Before delegating to the package manager, check whether a dedicated
        // vx provider explicitly declares it handles this `ecosystem:package` via
        // `ecosystem_aliases` in its provider.star.
        //
        // Example: cargo-audit declares
        //   ecosystem_aliases = [{"ecosystem": "cargo", "package": "audit"}]
        // so `vx cargo:audit` routes to the cargo-audit provider's pre-compiled
        // binary instead of `cargo install audit` (which fails — audit is a library).
        let provider_runtime_name = if let Some(colon_pos) = tool_spec.find(':') {
            let ecosystem = tool_spec[..colon_pos].split('@').next().unwrap_or("");
            let package_part = &tool_spec[colon_pos + 1..];
            let package = package_part
                .split('@')
                .next()
                .unwrap_or("")
                .split("::")
                .next()
                .unwrap_or("");
            let reg = vx_starlark::handle::global_registry().await;
            reg.get_runtime_for_ecosystem_package(ecosystem, package)
                .map(|s| s.to_owned())
        } else {
            None
        };

        if let Some(runtime_name) = provider_runtime_name {
            tracing::debug!(
                "ecosystem_aliases match: '{}' handles '{}', routing to provider binary",
                runtime_name,
                tool_spec
            );
            // Re-route: treat it as `vx cargo-audit [args]`, preserving any @version suffix
            let redirected_spec = if let Some(at_pos) = tool_spec.rfind('@') {
                let version_part = &tool_spec[at_pos..];
                format!("{}{}", runtime_name, version_part)
            } else {
                runtime_name
            };
            let request = RuntimeRequest::parse(&redirected_spec);
            return commands::execute::handle_with_deps(
                ctx.registry(),
                ctx.runtime_context(),
                &request.name,
                request.version.as_deref(),
                request.executable.as_deref(),
                &tool_args,
                ctx.use_system_path(),
                ctx.inherit_env(),
                ctx.cache_mode(),
                &with_deps,
            )
            .await;
        }

        tracing::debug!("Routing to execute_package_request");
        return execute_package_request(ctx, tool_spec, &tool_args, &with_deps).await;
    }

    // Parse as runtime request (supports runtime@version and runtime::executable syntax)
    let mut request = RuntimeRequest::parse(tool_spec);

    // When the `::` syntax is used (e.g. `msvc::ildasm`), check if the right-hand side
    // is itself a known runtime name.  If so, treat it as a direct runtime invocation
    // rather than an executable override — `vx msvc::ildasm` becomes `vx ildasm`.
    // This mirrors the logic in `where_cmd.rs`.
    if let Some(exe_override) = request.executable.clone() {
        let rhs_is_runtime = ctx.registry().get_runtime(&exe_override).is_some();
        if rhs_is_runtime {
            tracing::debug!(
                "execute_tool: '{}::{}' — rhs '{}' is a known runtime, redirecting",
                request.name,
                exe_override,
                exe_override
            );
            request.name = exe_override;
            request.executable = None;
        }
    }

    // Check if it's a known runtime first
    let is_known_runtime = ctx.registry().get_runtime(&request.name).is_some();

    // RFC 0033: If the runtime has a package_alias, route to package execution path
    // This makes `vx vite@5.0` equivalent to `vx npm:vite@5.0`
    if is_known_runtime && let Some(alias) = ctx.get_package_alias(&request.name) {
        let version_suffix = request
            .version
            .as_ref()
            .map(|v| format!("@{}", v))
            .unwrap_or_default();
        let executable_suffix = alias
            .executable
            .as_ref()
            .map(|e| format!("::{}", e))
            .unwrap_or_default();
        let pkg_spec = format!(
            "{}:{}{}{}",
            alias.ecosystem, alias.package, version_suffix, executable_suffix
        );
        tracing::debug!(
            "RFC 0033: Routing {} -> {} via package_alias",
            tool_spec,
            pkg_spec
        );
        return execute_package_request(ctx, &pkg_spec, &tool_args, &with_deps).await;
    }

    // If not a known runtime, try to execute as a globally installed package shim
    if !is_known_runtime
        && let Some(exit_code) =
            try_execute_global_shim(ctx, &request.name, &tool_args, &with_deps).await?
    {
        if exit_code != 0 {
            std::process::exit(exit_code);
        }
        return Ok(());
    }

    if !is_known_runtime && !request.is_shell_request() {
        ui::UI::tool_not_found(&request.name, &crate::registry::available_runtime_names());
        std::process::exit(1);
    }

    // Check if this is a shell request (runtime::shell syntax)
    if request.is_shell_request() {
        return execute_shell_request(ctx, &request, &tool_args, &with_deps).await;
    }

    // Execute as runtime with --with dependencies
    commands::execute::handle_with_deps(
        ctx.registry(),
        ctx.runtime_context(),
        &request.name,
        request.version.as_deref(),
        request.executable.as_deref(),
        &tool_args,
        ctx.use_system_path(),
        ctx.inherit_env(),
        ctx.cache_mode(),
        &with_deps,
    )
    .await
}

/// Execute a shell request (runtime::shell syntax)
///
/// This launches an interactive shell with the runtime's environment configured.
/// Examples:
/// - `vx git::git-bash` - Launch Git Bash with git's environment
/// - `vx node::cmd` - Launch cmd with node's environment
/// - `vx go::powershell` - Launch PowerShell with go's environment
async fn execute_shell_request(
    ctx: &CommandContext,
    request: &RuntimeRequest,
    args: &[String],
    with_deps: &[WithDependency],
) -> Result<()> {
    let shell_name = request.shell_name().unwrap_or("cmd");
    let version = request.version.as_deref().unwrap_or("latest");

    ui::UI::info(&format!(
        "Launching {} shell with {} environment...",
        shell_name, request.name
    ));

    // Ensure the runtime is installed first
    if let Some(runtime) = ctx.registry().get_runtime(&request.name)
        && !runtime
            .is_installed(version, ctx.runtime_context())
            .await
            .unwrap_or(false)
    {
        ui::UI::info(&format!(
            "Runtime '{}@{}' is not installed. Installing...",
            request.name, version
        ));
        ensure_runtime_installed_for_ecosystem(ctx, &request.name).await?;
    }

    // Try to get shell path from the runtime first
    let shell_exe = if let Some(runtime) = ctx.registry().get_runtime(&request.name) {
        if let Some(shell_path) = runtime.get_shell_path(shell_name, version, ctx.runtime_context())
        {
            ui::UI::debug(&format!(
                "Runtime '{}' provides shell path: {}",
                request.name,
                shell_path.display()
            ));
            shell_path
        } else {
            // Runtime doesn't provide this shell, search in system PATH
            find_shell_in_path(shell_name)?
        }
    } else {
        // Runtime not found, search in system PATH
        find_shell_in_path(shell_name)?
    };

    ui::UI::debug(&format!("Using shell: {}", shell_exe.display()));

    // Build environment with runtime's tools
    let mut tool_env = vx_env::ToolEnvironment::new();

    // Add the runtime
    tool_env = tool_env.tool(&request.name, version);

    // Add --with dependencies
    for dep in with_deps {
        let dep_version = dep.version.as_deref().unwrap_or("latest");
        tool_env = tool_env.tool(&dep.runtime, dep_version);
    }

    let env = tool_env
        .include_vx_bin(true)
        .inherit_path(true)
        .warn_missing(false)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build shell environment: {}", e))?;

    ui::UI::debug(&format!(
        "Built shell environment with {} entries",
        env.len()
    ));

    // Build shell command args
    // For shells that open new windows by default (like git-bash), we need to
    // add flags to attach to the current terminal instead of opening a new window.
    let shell_args: Vec<String> = if args.is_empty() {
        build_default_shell_args(shell_name)
    } else {
        args.to_vec()
    };

    ui::UI::debug(&format!(
        "Launching shell {} with args: {:?}",
        shell_exe.display(),
        shell_args
    ));

    // Execute the shell
    let status = std::process::Command::new(&shell_exe)
        .args(&shell_args)
        .envs(&env)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to launch shell '{}': {}", shell_name, e))?;

    let exit_code = status.code().unwrap_or(1);
    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}

/// Find a shell executable in system PATH
///
/// This searches for the shell with platform-specific extensions:
/// - Windows: .exe, .cmd, .bat, and without extension
/// - Unix: just the name
fn find_shell_in_path(shell_name: &str) -> Result<std::path::PathBuf> {
    // On Windows, try multiple extensions
    #[cfg(windows)]
    {
        let extensions = [".exe", ".cmd", ".bat", ""];
        for ext in extensions {
            let name_with_ext = format!("{}{}", shell_name, ext);
            if let Ok(path) = which::which(&name_with_ext) {
                return Ok(path);
            }
        }
        Err(anyhow::anyhow!(
            "Shell '{}' not found in system PATH. Tried extensions: {:?}",
            shell_name,
            extensions
        ))
    }

    #[cfg(not(windows))]
    {
        which::which(shell_name).map_err(|_| {
            anyhow::anyhow!(
                "Shell '{}' not found in system PATH. Please install it first.",
                shell_name
            )
        })
    }
}

/// Build default shell arguments to run interactively in the current terminal
///
/// Different shells have different flags to run interactively:
///
/// - `git-bash`: We use `bin/bash.exe` (not `git-bash.exe` which is a MinTTY launcher).
///   `bin/bash.exe` runs directly in the current terminal, same as VSCode does.
///   Pass `--init-file` to load Git's shell integration, or just `-i` for interactive mode.
/// - `bash` / `sh` / `zsh` / `fish`: Run in current terminal by default
/// - `cmd`: No extra flags needed
/// - `powershell` / `pwsh`: No extra flags needed
fn build_default_shell_args(shell_name: &str) -> Vec<String> {
    match shell_name.to_lowercase().as_str() {
        // git-bash: we use bin/bash.exe directly (like VSCode does).
        // bin/bash.exe runs in the current terminal without opening a new MinTTY window.
        // Pass --login -i to get a proper interactive login shell with Git environment.
        "git-bash" => {
            vec!["--login".to_string(), "-i".to_string()]
        }
        // bash, sh, zsh, fish, etc. run in current terminal by default
        _ => vec![],
    }
}

/// Execute an RFC 0027 package request
///
/// Syntax: `<ecosystem>[@runtime_version]:<package>[@version][::executable]`
///
/// This function implements uvx/npx-like behavior:
/// - If the package is already installed, execute it directly
/// - If not installed, auto-install it first, then execute
async fn execute_package_request(
    ctx: &CommandContext,
    spec: &str,
    args: &[String],
    with_deps: &[WithDependency],
) -> Result<()> {
    let pkg_request = PackageRequest::parse(spec)?;

    let paths = ctx.runtime_context().paths.clone();

    // If --with dependencies are specified, we need to use ShimExecutor with custom env
    // For now, we auto-install --with deps first, then execute with the enhanced environment
    if !with_deps.is_empty() {
        // Auto-install --with dependencies
        for dep in with_deps {
            if let Some(runtime) = ctx.registry().get_runtime(&dep.runtime) {
                let version = dep.version.as_deref().unwrap_or("latest");
                if !runtime
                    .is_installed(version, ctx.runtime_context())
                    .await
                    .unwrap_or(false)
                {
                    ui::UI::info(&format!(
                        "--with dependency '{}@{}' is not installed. Installing...",
                        dep.runtime, version
                    ));
                    ensure_runtime_installed_for_ecosystem(ctx, &dep.runtime).await?;
                }
            }
        }
    }

    let executor = ShimExecutor::new(paths.packages_registry_file(), paths.shims_dir());

    match executor
        .execute_request_with_deps(&pkg_request, args, with_deps)
        .await
    {
        Ok(exit_code) => {
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
            Ok(())
        }
        Err(vx_shim::ShimError::PackageNotInstalled { ecosystem, package }) => {
            // Package not installed - auto-install it (uvx/npx behavior)
            ui::UI::info(&format!(
                "Package '{}:{}' is not installed. Installing...",
                ecosystem, package
            ));

            // Auto-install the package
            auto_install_package(ctx, &pkg_request).await?;

            // Retry execution after installation
            let executor = ShimExecutor::new(paths.packages_registry_file(), paths.shims_dir());
            match executor
                .execute_request_with_deps(&pkg_request, args, with_deps)
                .await
            {
                Ok(exit_code) => {
                    if exit_code != 0 {
                        std::process::exit(exit_code);
                    }
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e.into()),
    }
}

/// Auto-install a package for RFC 0027 implicit execution
///
/// This replicates the logic from `vx global install` but without user prompts.
async fn auto_install_package(ctx: &CommandContext, pkg_request: &PackageRequest) -> Result<()> {
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = paths.packages_registry_file();
    let mut registry = PackageRegistry::load_or_create(&registry_path)?;

    let ecosystem = &pkg_request.ecosystem;
    let package = &pkg_request.package;
    let version = pkg_request.version.as_deref().unwrap_or("latest");

    // Ensure ALL runtime dependencies are installed (auto-install if needed)
    // Some npm packages may use bun internally, so we install both node and bun
    let runtime_installed = match get_all_required_runtimes_for_ecosystem(ecosystem) {
        runtimes if !runtimes.is_empty() => {
            let mut any_installed = false;
            for runtime in runtimes {
                match ensure_runtime_installed_for_ecosystem(ctx, runtime).await {
                    Ok(true) => any_installed = true,
                    Ok(false) => {}
                    Err(e) => {
                        // For optional runtimes like bun, just log and continue
                        if runtime != "node" && runtime != "uv" && runtime != "go" {
                            tracing::debug!("Optional runtime {} not installed: {}", runtime, e);
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            any_installed
        }
        _ => false,
    };

    // Get the appropriate installer for this ecosystem
    // For npm/npx ecosystem, prefer bun (faster) with npm as fallback
    // For uvx ecosystem, we need to use the uv executable from the installed uv
    let installer: Box<dyn vx_ecosystem_pm::EcosystemInstaller> = match ecosystem.as_str() {
        "npm" | "node" | "npx" => {
            // Prefer bun if available (faster), fall back to npm
            // Try bun from vx store first
            let bun_path = if runtime_installed {
                match vx_paths::get_bundled_tool_path("bun", "bun") {
                    Ok(Some(path)) if path.exists() => Some(path),
                    _ => None,
                }
            } else {
                None
            };

            if let Some(path) = bun_path {
                tracing::debug!("Using bun (preferred) from: {}", path.display());
                Box::new(vx_ecosystem_pm::installers::BunInstaller::with_bun_path(
                    path,
                ))
            } else {
                // Bun not in vx store, check system PATH
                let bun_installer = vx_ecosystem_pm::installers::BunInstaller::new();
                if bun_installer.is_available() {
                    tracing::debug!("Using bun (preferred) from system PATH");
                    Box::new(bun_installer)
                } else {
                    // Fall back to npm
                    let npm_path = if runtime_installed {
                        match vx_paths::get_bundled_tool_path("node", "npm") {
                            Ok(Some(path)) if path.exists() => Some(path),
                            _ => None,
                        }
                    } else {
                        None
                    };

                    if let Some(path) = npm_path {
                        tracing::debug!("Using npm (fallback) from: {}", path.display());
                        Box::new(vx_ecosystem_pm::installers::NpmInstaller::with_npm_path(
                            path,
                        ))
                    } else {
                        Box::new(vx_ecosystem_pm::installers::NpmInstaller::new())
                    }
                }
            }
        }
        "uvx" => {
            // For uvx ecosystem, try to find uv executable from the vx-installed uv
            // Use vx-paths RuntimeRoot to get the uv executable path
            let uv_path = match vx_paths::get_bundled_tool_path("uv", "uv") {
                Ok(Some(path)) if path.exists() => Some(path),
                _ => None,
            };

            if let Some(path) = uv_path {
                tracing::debug!("Using uv from vx store: {}", path.display());
                Box::new(vx_ecosystem_pm::installers::UvxInstaller::with_uv_path(
                    path,
                ))
            } else {
                Box::new(vx_ecosystem_pm::installers::UvxInstaller::new())
            }
        }
        "deno" => {
            // For deno ecosystem, try to find deno executable from the vx-installed deno
            // Use vx-paths RuntimeRoot to get the deno executable path
            let deno_path = match vx_paths::get_bundled_tool_path("deno", "deno") {
                Ok(Some(path)) if path.exists() => Some(path),
                _ => None,
            };

            if let Some(path) = deno_path {
                tracing::debug!("Using deno from vx store: {}", path.display());
                Box::new(vx_ecosystem_pm::installers::DenoInstaller::with_deno_path(
                    path,
                ))
            } else {
                Box::new(vx_ecosystem_pm::installers::DenoInstaller::new())
            }
        }
        _ => get_installer(ecosystem)
            .with_context(|| format!("Unsupported ecosystem: {}", ecosystem))?,
    };

    // Build install options (non-verbose, non-force for auto-install)
    let options = InstallOptions {
        force: false,
        verbose: false,
        runtime_version: pkg_request.runtime_spec.as_ref().map(|s| s.version.clone()),
        extra_args: Vec::new(),
    };

    // Get the installation directory
    let install_dir = paths.global_package_dir(ecosystem, package, version);

    // Perform the actual installation
    let result = installer
        .install(&install_dir, package, version, &options)
        .await
        .with_context(|| format!("Failed to install {}:{}@{}", ecosystem, package, version))?;

    // Create GlobalPackage from EcosystemInstallResult for registry
    let global_package = GlobalPackage::new(
        package.clone(),
        result.version.clone(),
        ecosystem.clone(),
        result.install_dir.clone(),
    )
    .with_executables(result.executables.clone());

    // Register package
    registry.register(global_package);
    registry.save(&registry_path)?;

    ui::UI::success(&format!(
        "Installed {}:{}@{}",
        ecosystem, package, result.version
    ));

    // Create shims for package executables
    let shims_dir = paths.shims_dir();
    let bin_dir = result.bin_dir.clone();

    for exe in &result.executables {
        let exe_path = bin_dir.join(if cfg!(windows) {
            format!("{}.exe", exe)
        } else {
            exe.to_string()
        });

        // Try with the extension first, then without on Windows
        let target_path = if exe_path.exists() {
            exe_path
        } else {
            bin_dir.join(exe)
        };

        if target_path.exists()
            && let Err(e) = shims::create_shim(&shims_dir, exe, &target_path)
        {
            ui::UI::warn(&format!("Failed to create shim for {}: {}", exe, e));
        }
    }

    Ok(())
}

/// Get all required runtimes for an ecosystem (including optional ones)
///
/// Some npm packages may use bun internally, so we install both node and bun
/// to ensure maximum compatibility.
fn get_all_required_runtimes_for_ecosystem(ecosystem: &str) -> Vec<&'static str> {
    match ecosystem.to_lowercase().as_str() {
        // Node.js ecosystem - node is required, bun is optional but recommended
        // Some packages like opencode use bun internally
        // npx is bundled with npm/node, so it also requires node
        "npm" | "node" | "yarn" | "pnpm" | "npx" => vec!["node", "bun"],
        // Bun ecosystem just needs bun; bunx is bun's package runner
        "bun" | "bunx" => vec!["bun"],
        // dlx: pnpm's oneshot runner, requires node (for pnpm) and pnpm itself
        "dlx" => vec!["node"],
        // Python ecosystem requires uv
        "pip" | "python" | "pypi" => vec!["uv"],
        "uv" => vec![], // uv is self-contained
        // uvx: Python CLI tools run via uvx (isolated environments), requires uv
        "uvx" => vec!["uv"],
        // Deno ecosystem        // Deno ecosystem - deno is self-contained
        "deno" => vec!["deno"],
        // .NET ecosystem - requires dotnet SDK
        "dotnet-tool" | "dotnet" => vec!["dotnet"],
        // Java ecosystem - jbang requires java
        "jbang" | "java" => vec!["java"],
        // Rust ecosystem
        "cargo" | "rust" | "crates" => vec!["cargo"],
        // Go ecosystem
        "go" | "golang" => vec!["go"],
        // Ruby ecosystem
        "gem" | "ruby" | "rubygems" => vec!["ruby"],
        // Windows ecosystem (choco is self-contained)
        "choco" | "chocolatey" => vec!["choco"],
        _ => vec![],
    }
}

/// Ensure a runtime is installed for ecosystem package installation
///
/// Returns `true` if the runtime is installed (either was already installed or was just installed).
async fn ensure_runtime_installed_for_ecosystem(
    ctx: &CommandContext,
    runtime_name: &str,
) -> Result<bool> {
    // Check if runtime is already available
    if let Some(runtime) = ctx.registry().get_runtime(runtime_name) {
        let context = ctx.runtime_context();

        // Check if already installed
        match runtime.is_installed("latest", context).await {
            Ok(true) => {
                tracing::debug!("Runtime {} is already installed", runtime_name);
                return Ok(true);
            }
            Ok(false) => {
                // Not installed, try to auto-install
                ui::UI::info(&format!(
                    "Runtime {} is not installed. Auto-installing...",
                    runtime_name
                ));
            }
            Err(e) => {
                tracing::warn!("Failed to check if {} is installed: {}", runtime_name, e);
            }
        }

        // Fetch versions to get the latest
        ui::UI::info(&format!("Fetching versions for {}...", runtime_name));
        let versions = match runtime.fetch_versions(context).await {
            Ok(v) => v,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to fetch versions for {}: {}. Please install it manually with 'vx install {}'.",
                    runtime_name,
                    e,
                    runtime_name
                ));
            }
        };

        let version = versions
            .iter()
            .find(|v| !v.prerelease)
            .map(|v| v.version.clone())
            .or_else(|| versions.first().map(|v| v.version.clone()))
            .ok_or_else(|| anyhow::anyhow!("No versions found for {}", runtime_name))?;

        ui::UI::info(&format!("Installing {} {}...", runtime_name, version));

        // Run pre-install hook
        runtime.pre_install(&version, context).await?;

        // Install the runtime
        let result = runtime.install(&version, context).await?;

        // Verify the installation
        if !context.fs.exists(&result.executable_path) {
            return Err(anyhow::anyhow!(
                "Installation completed but executable not found at {}",
                result.executable_path.display()
            ));
        }

        // Run post-install hook
        runtime.post_install(&version, context).await?;

        ui::UI::success(&format!(
            "Successfully installed {} {}",
            runtime_name, version
        ));

        Ok(true)
    } else {
        Err(anyhow::anyhow!(
            "Runtime {} not found in registry. Cannot auto-install.",
            runtime_name
        ))
    }
}

/// Try to execute a globally installed package's executable (shim)
///
/// Returns:
/// - `Ok(Some(exit_code))` if a shim was found and executed
/// - `Ok(None)` if no shim was found
/// - `Err(...)` if an error occurred during execution
async fn try_execute_global_shim(
    ctx: &CommandContext,
    exe_name: &str,
    args: &[String],
    with_deps: &[WithDependency],
) -> Result<Option<i32>> {
    let paths = ctx.runtime_context().paths.clone();
    let executor = ShimExecutor::new(paths.packages_registry_file(), paths.shims_dir());

    // If --with dependencies are specified, auto-install them first
    if !with_deps.is_empty() {
        for dep in with_deps {
            if let Some(runtime) = ctx.registry().get_runtime(&dep.runtime) {
                let version = dep.version.as_deref().unwrap_or("latest");
                if !runtime
                    .is_installed(version, ctx.runtime_context())
                    .await
                    .unwrap_or(false)
                {
                    ui::UI::info(&format!(
                        "--with dependency '{}@{}' is not installed. Installing...",
                        dep.runtime, version
                    ));
                    ensure_runtime_installed_for_ecosystem(ctx, &dep.runtime).await?;
                }
            }
        }
    }

    match executor
        .try_execute_with_deps(exe_name, args, with_deps)
        .await
    {
        Ok(result) => Ok(result),
        Err(e) => {
            ui::UI::warn(&format!("Shim execution warning: {}", e));
            Ok(None)
        }
    }
}

/// Main CLI application structure (kept for backwards compatibility)
pub struct VxCli {
    ctx: CommandContext,
}

impl VxCli {
    /// Create a new VxCli instance with the given provider registry
    pub fn new(registry: ProviderRegistry, context: vx_runtime::RuntimeContext) -> Self {
        let ctx = CommandContext::new(registry, context, GlobalOptions::default());
        Self { ctx }
    }

    /// Run the CLI application
    pub async fn run(self) -> Result<()> {
        let cli = Cli::parse();
        self.run_with_cli(cli).await
    }

    /// Run the CLI application with pre-parsed CLI arguments
    pub async fn run_with_cli(self, cli: Cli) -> Result<()> {
        // Update context with CLI flags
        let options = GlobalOptions::from(&cli);
        let ctx = CommandContext::new_with_arc(
            self.ctx.registry.clone(),
            self.ctx.runtime_context.clone(),
            options,
        );

        // Route to appropriate handler
        match &cli.command {
            Some(command) => command.execute(&ctx).await,
            None => {
                if cli.args.is_empty() {
                    Cli::parse_from(["vx", "--help"]);
                    Ok(())
                } else {
                    execute_tool(&ctx, &cli.args, &cli.with_deps).await
                }
            }
        }
    }
}
