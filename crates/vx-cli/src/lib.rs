//! VX CLI - Command Line Interface for VX Tool Manager

use anyhow::Result;
use clap::Parser;
use vx_runtime::ProviderRegistry;

pub mod cli;
pub mod commands;
pub mod registry;
pub mod tracing_setup;
pub mod ui;

#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
mod cli_tests;

#[cfg(test)]
mod plugin_tests;

// Re-export for convenience
pub use cli::Cli;
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

    // Create runtime context
    let context = create_context()?;

    // Create and run CLI with pre-parsed args
    let vx_cli = VxCli::new(registry, context);
    vx_cli.run_with_cli(cli).await
}

/// Main CLI application structure
pub struct VxCli {
    registry: ProviderRegistry,
    context: vx_runtime::RuntimeContext,
}

impl VxCli {
    /// Create a new VxCli instance with the given provider registry
    pub fn new(registry: ProviderRegistry, context: vx_runtime::RuntimeContext) -> Self {
        Self { registry, context }
    }

    /// Run the CLI application
    pub async fn run(self) -> Result<()> {
        let cli = Cli::parse();
        self.run_with_cli(cli).await
    }

    /// Run the CLI application with pre-parsed CLI arguments
    pub async fn run_with_cli(self, cli: Cli) -> Result<()> {
        // Handle global flags
        if cli.verbose {
            // Verbose logging is already set up in tracing_setup
        }

        // Route to appropriate command handler
        match &cli.command {
            Some(command) => self.handle_command(command.clone(), &cli).await,
            None => {
                // No subcommand provided, try to execute as tool
                if cli.args.is_empty() {
                    // Show help if no arguments
                    Cli::parse_from(["vx", "--help"]);
                    Ok(())
                } else {
                    // Execute tool
                    self.execute_tool(&cli.args, cli.use_system_path).await
                }
            }
        }
    }

    /// Handle a specific command
    async fn handle_command(&self, command: cli::Commands, _cli: &Cli) -> Result<()> {
        use cli::Commands;

        match command {
            Commands::Version => commands::version::handle().await,
            Commands::List {
                tool,
                status,
                installed: _,
                available: _,
            } => {
                commands::list::handle(&self.registry, &self.context, tool.as_deref(), status).await
            }
            Commands::Install {
                tool,
                version,
                force,
            } => {
                commands::install::handle(
                    &self.registry,
                    &self.context,
                    &tool,
                    version.as_deref(),
                    force,
                )
                .await
            }
            Commands::Update { tool, apply } => {
                commands::update::handle(&self.registry, &self.context, tool.as_deref(), apply)
                    .await
            }

            Commands::SelfUpdate {
                check,
                version: _,
                token,
                prerelease,
                force,
            } => commands::self_update::handle(token.as_deref(), prerelease, force, check).await,

            Commands::Uninstall {
                tool,
                version,
                force,
            } => {
                commands::remove::handle(
                    &self.registry,
                    &self.context,
                    &tool,
                    version.as_deref(),
                    force,
                )
                .await
            }

            Commands::Which { tool, all } => {
                commands::where_cmd::handle(&self.registry, &tool, all).await
            }

            Commands::Versions {
                tool,
                latest,
                prerelease,
                detailed,
                interactive,
            } => {
                commands::fetch::handle(
                    &self.registry,
                    &self.context,
                    &tool,
                    latest,
                    detailed,
                    interactive,
                    prerelease,
                )
                .await
            }
            Commands::Switch {
                tool_version,
                global,
            } => commands::switch::handle(&self.registry, &tool_version, global).await,
            Commands::Config { command } => match command {
                Some(cli::ConfigCommand::Show) | None => commands::config::handle().await,
                Some(cli::ConfigCommand::Set { key, value }) => {
                    commands::config::handle_set(&key, &value).await
                }
                Some(cli::ConfigCommand::Get { key }) => commands::config::handle_get(&key).await,
                Some(cli::ConfigCommand::Reset { key }) => {
                    commands::config::handle_reset(key.clone()).await
                }
                Some(cli::ConfigCommand::Edit) => commands::config::handle_edit().await,
            },
            Commands::Init {
                interactive,
                template,
                tools,
                force,
                dry_run,
                list_templates,
            } => {
                commands::init::handle(interactive, template, tools, force, dry_run, list_templates)
                    .await
            }

            Commands::Clean {
                dry_run,
                cache,
                orphaned,
                all,
                force,
                older_than,
                verbose,
            } => {
                // Map new clean options to cleanup options
                let cache_only = cache && !all;
                let orphaned_only = orphaned && !all;
                commands::cleanup::handle(
                    dry_run,
                    cache_only,
                    orphaned_only,
                    force,
                    older_than,
                    verbose,
                )
                .await
            }
            Commands::Stats => commands::stats::handle(&self.registry).await,
            Commands::Plugin { command } => commands::plugin::handle(&self.registry, command).await,
            Commands::Venv { command } => commands::venv_cmd::handle(command).await,
            Commands::Global { command } => commands::global::handle(command).await,
            Commands::Env { command } => commands::env::handle(command).await,
            Commands::Search {
                query,
                category,
                installed_only,
                available_only,
                format,
                verbose,
            } => {
                commands::search::handle(
                    &self.registry,
                    query,
                    category,
                    installed_only,
                    available_only,
                    format,
                    verbose,
                )
                .await
            }
            Commands::Sync {
                check,
                force,
                dry_run,
                verbose,
                no_parallel,
                no_auto_install,
            } => {
                commands::sync::handle(
                    &self.registry,
                    check,
                    force,
                    dry_run,
                    verbose,
                    no_parallel,
                    no_auto_install,
                )
                .await
            }

            Commands::Shell { command } => {
                use crate::cli::ShellCommand;
                match command {
                    ShellCommand::Init { shell } => {
                        commands::shell::handle_shell_init(shell.clone()).await
                    }
                    ShellCommand::Completions { shell } => {
                        commands::shell::handle_completion(shell.clone()).await
                    }
                }
            }
        }
    }

    /// Execute a tool with the given arguments
    async fn execute_tool(&self, args: &[String], use_system_path: bool) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("No tool specified"));
        }

        let tool_name = &args[0];
        let tool_args = &args[1..];

        commands::execute::handle(
            &self.registry,
            &self.context,
            tool_name,
            tool_args,
            use_system_path,
        )
        .await
    }
}
