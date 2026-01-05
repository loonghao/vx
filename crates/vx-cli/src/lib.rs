//! VX CLI - Command Line Interface for VX Tool Manager

use anyhow::Result;
use clap::Parser;
use vx_resolver::RuntimeRequest;
use vx_runtime::ProviderRegistry;

pub mod cli;
pub mod commands;
pub mod config;
pub mod registry;
pub mod suggestions;
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
/// Supports `runtime@version` syntax:
/// - `vx yarn@1.21.1 global add terminalizer`
/// - `vx node@20 --version`
async fn execute_tool(ctx: &CommandContext, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("No tool specified"));
    }

    // Parse the runtime request (supports runtime@version syntax)
    let request = RuntimeRequest::parse(&args[0]);
    let tool_args = &args[1..];

    commands::execute::handle_with_version(
        ctx.registry(),
        ctx.runtime_context(),
        &request.name,
        request.version.as_deref(),
        tool_args,
        ctx.use_system_path(),
        ctx.cache_mode(),
    )
    .await
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
