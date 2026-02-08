//! VX CLI - Command Line Interface for VX Tool Manager

use anyhow::{Context, Result};
use clap::Parser;
use vx_core::WithDependency;
use vx_ecosystem_pm::{get_installer, InstallOptions};
use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_paths::shims;
use vx_resolver::RuntimeRequest;
use vx_runtime::{init_constraints_from_manifests, ProviderRegistry};
use vx_shim::{PackageRequest, ShimExecutor};

pub mod cli;
pub mod commands;
pub mod config;
pub mod error_handler;
pub mod registry;
pub mod suggestions;
pub mod system_tools;
pub mod tracing_setup;
pub mod ui;

#[cfg(test)]
pub mod test_utils;

// Re-export for convenience
pub use cli::Cli;
pub use commands::{CommandContext, CommandHandler, GlobalOptions};
pub use registry::{create_context, create_registry, ProviderRegistryExt};
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

    // Initialize constraints registry from embedded provider manifests
    // This makes manifest-defined dependency constraints available globally.
    let _ = init_constraints_from_manifests(registry::get_embedded_manifests().iter().copied());

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

    tracing::debug!(
        "execute_tool: tool_spec={}, is_package_request={}, with_deps={:?}",
        tool_spec,
        PackageRequest::is_package_request(tool_spec),
        with_deps
    );

    // Check if this is an RFC 0027 package request (ecosystem:package syntax)
    if PackageRequest::is_package_request(tool_spec) {
        tracing::debug!("Routing to execute_package_request");
        return execute_package_request(ctx, tool_spec, &tool_args, &with_deps).await;
    }

    // Parse as runtime request (supports runtime@version and runtime::executable syntax)
    let request = RuntimeRequest::parse(tool_spec);

    // Check if it's a known runtime first
    let is_known_runtime = ctx.registry().get_runtime(&request.name).is_some();

    // If not a known runtime, try to execute as a globally installed package shim
    if !is_known_runtime {
        if let Some(exit_code) =
            try_execute_global_shim(ctx, &request.name, &tool_args, &with_deps).await?
        {
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
            return Ok(());
        }
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
    // For npm ecosystem, we need to use the npm executable from the installed node
    let installer: Box<dyn vx_ecosystem_pm::EcosystemInstaller> = match ecosystem.as_str() {
        "npm" | "node" => {
            // For npm ecosystem, try to find npm executable from the installed node
            // Use vx-paths RuntimeRoot to get bundled tool path
            let npm_path = if runtime_installed {
                match vx_paths::get_bundled_tool_path("node", "npm") {
                    Ok(Some(path)) if path.exists() => Some(path),
                    _ => None,
                }
            } else {
                None
            };

            if let Some(path) = npm_path {
                tracing::debug!("Using npm from: {}", path.display());
                Box::new(vx_ecosystem_pm::installers::NpmInstaller::with_npm_path(
                    path,
                ))
            } else {
                Box::new(vx_ecosystem_pm::installers::NpmInstaller::new())
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

        if target_path.exists() {
            if let Err(e) = shims::create_shim(&shims_dir, exe, &target_path) {
                ui::UI::warn(&format!("Failed to create shim for {}: {}", exe, e));
            }
        }
    }

    Ok(())
}

/// Get the required runtime for an ecosystem (primary dependency only)
///
/// Note: This is kept for reference but the `get_all_required_runtimes_for_ecosystem`
/// is now used for package installation to support packages that use multiple runtimes.
#[allow(dead_code)]
fn get_required_runtime_for_ecosystem(ecosystem: &str) -> Option<&'static str> {
    match ecosystem.to_lowercase().as_str() {
        // Node.js ecosystem requires node (which provides npm)
        "npm" | "node" | "yarn" | "pnpm" | "bun" => Some("node"),
        // Python ecosystem requires uv or python
        "pip" | "python" | "pypi" => Some("uv"),
        "uv" => None, // uv is self-contained
        // Rust ecosystem
        "cargo" | "rust" | "crates" => Some("cargo"),
        // Go ecosystem
        "go" | "golang" => Some("go"),
        // Ruby ecosystem
        "gem" | "ruby" | "rubygems" => Some("ruby"),
        _ => None,
    }
}

/// Get ALL required runtimes for an ecosystem (including optional ones)
///
/// Some npm packages may use bun internally, so we install both node and bun
/// to ensure maximum compatibility.
fn get_all_required_runtimes_for_ecosystem(ecosystem: &str) -> Vec<&'static str> {
    match ecosystem.to_lowercase().as_str() {
        // Node.js ecosystem - node is required, bun is optional but recommended
        // Some packages like opencode use bun internally
        "npm" | "node" | "yarn" | "pnpm" => vec!["node", "bun"],
        // Bun ecosystem just needs bun
        "bun" => vec!["bun"],
        // Python ecosystem requires uv
        "pip" | "python" | "pypi" => vec!["uv"],
        "uv" => vec![], // uv is self-contained
        // Rust ecosystem
        "cargo" | "rust" | "crates" => vec!["cargo"],
        // Go ecosystem
        "go" | "golang" => vec!["go"],
        // Ruby ecosystem
        "gem" | "ruby" | "rubygems" => vec!["ruby"],
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
