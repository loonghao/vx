//! VX CLI - Command Line Interface for VX Tool Manager

use anyhow::{Context, Result};
use clap::Parser;
use vx_ecosystem_pm::{get_installer, InstallOptions};
use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_paths::shims;
use vx_resolver::RuntimeRequest;
use vx_runtime::{init_constraints_from_manifests, ProviderRegistry};
use vx_shim::{PackageRequest, ShimExecutor};

pub mod cli;
pub mod commands;
pub mod config;
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

    // Setup tracing with debug mode if requested
    if cli.debug {
        tracing_setup::setup_tracing_with_debug(true);
    } else {
        setup_tracing();
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
    match &cli.command {
        Some(command) => command.execute(&cmd_ctx).await,
        None => {
            // No subcommand provided, try to execute as tool
            if cli.args.is_empty() {
                // Show help if no arguments
                Cli::parse_from(["vx", "--help"]);
                Ok(())
            } else {
                // Execute tool
                execute_tool(&cmd_ctx, &cli.args).await
            }
        }
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
async fn execute_tool(ctx: &CommandContext, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("No tool specified"));
    }

    let tool_spec = &args[0];
    let tool_args: Vec<String> = args[1..].to_vec();

    // Check if this is an RFC 0027 package request (ecosystem:package syntax)
    if PackageRequest::is_package_request(tool_spec) {
        return execute_package_request(ctx, tool_spec, &tool_args).await;
    }

    // Parse as runtime request (supports runtime@version syntax)
    let request = RuntimeRequest::parse(tool_spec);

    // Check if it's a known runtime first
    let is_known_runtime = ctx.registry().get_runtime(&request.name).is_some();

    // If not a known runtime, try to execute as a globally installed package shim
    if !is_known_runtime {
        if let Some(exit_code) = try_execute_global_shim(ctx, &request.name, &tool_args).await? {
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
            return Ok(());
        }
    }

    // Execute as runtime
    commands::execute::handle_with_version(
        ctx.registry(),
        ctx.runtime_context(),
        &request.name,
        request.version.as_deref(),
        &tool_args,
        ctx.use_system_path(),
        ctx.inherit_env(),
        ctx.cache_mode(),
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
) -> Result<()> {
    let pkg_request = PackageRequest::parse(spec)?;

    let paths = ctx.runtime_context().paths.clone();
    let executor = ShimExecutor::new(paths.packages_registry_file(), paths.shims_dir());

    match executor.execute_request(&pkg_request, args).await {
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
            match executor.execute_request(&pkg_request, args).await {
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

    // Get the appropriate installer for this ecosystem
    let installer = get_installer(ecosystem)
        .with_context(|| format!("Unsupported ecosystem: {}", ecosystem))?;

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
) -> Result<Option<i32>> {
    let paths = ctx.runtime_context().paths.clone();
    let executor = ShimExecutor::new(paths.packages_registry_file(), paths.shims_dir());

    match executor.try_execute(exe_name, args).await {
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
                    execute_tool(&ctx, &cli.args).await
                }
            }
        }
    }
}
