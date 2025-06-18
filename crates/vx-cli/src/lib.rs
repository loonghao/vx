//! VX CLI - Command Line Interface for VX Tool Manager

use anyhow::Result;
use clap::Parser;
use vx_core::{PluginRegistry, VxError};

pub mod cli;
pub mod commands;
pub mod tracing_setup;
pub mod ui;

// Re-export for convenience
pub use cli::Cli;
pub use tracing_setup::setup_tracing;

/// Main entry point for the VX CLI application
/// This function sets up the plugin registry and runs the CLI
pub async fn main() -> anyhow::Result<()> {
    // Setup tracing
    setup_tracing();

    // Create plugin registry with all available plugins
    let mut registry = vx_core::PluginRegistry::new();

    // Register Node.js plugin
    let _ = registry.register(Box::new(vx_tool_node::NodePlugin::new()));

    // Register Go plugin
    let _ = registry.register(Box::new(vx_tool_go::GoPlugin::new()));

    // Register Rust plugin
    let _ = registry.register(Box::new(vx_tool_rust::RustPlugin::new()));

    // Register UV plugin
    let _ = registry.register(Box::new(vx_tool_uv::UvPlugin::new()));

    // Create and run CLI
    let cli = VxCli::new(registry);
    cli.run().await
}

/// Main CLI application structure
pub struct VxCli {
    registry: PluginRegistry,
}

impl VxCli {
    /// Create a new VxCli instance with the given plugin registry
    pub fn new(registry: PluginRegistry) -> Self {
        Self { registry }
    }

    /// Run the CLI application
    pub async fn run(self) -> Result<()> {
        let cli = Cli::parse();

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
            Commands::Version => commands::version::handle().await.map_err(Into::into),
            Commands::List {
                tool,
                status,
                installed: _,
                available: _,
            } => commands::list::handle(&self.registry, tool.as_deref(), status)
                .await
                .map_err(Into::into),
            Commands::Install {
                tool,
                version,
                force,
            } => commands::install::handle(&self.registry, &tool, version.as_deref(), force)
                .await
                .map_err(Into::into),
            Commands::Update { tool, apply } => {
                commands::update::handle(&self.registry, tool.as_deref(), apply)
                    .await
                    .map_err(Into::into)
            }

            Commands::SelfUpdate { check, version } => {
                commands::self_update::handle(check, version.as_deref())
                    .await
                    .map_err(Into::into)
            }

            Commands::Uninstall {
                tool,
                version,
                force,
            } => commands::remove::handle(&self.registry, &tool, version.as_deref(), force)
                .await
                .map_err(Into::into),

            Commands::Which { tool, all } => {
                commands::where_cmd::handle(&self.registry, &tool, all)
                    .await
                    .map_err(Into::into)
            }

            Commands::Versions {
                tool,
                latest,
                prerelease,
                detailed,
                interactive,
            } => commands::fetch::handle(
                &self.registry,
                &tool,
                latest,
                detailed,
                interactive,
                prerelease,
            )
            .await
            .map_err(Into::into),
            Commands::Switch {
                tool_version,
                global,
            } => commands::switch::handle(&self.registry, &tool_version, global)
                .await
                .map_err(Into::into),
            Commands::Config { command } => match command {
                Some(cli::ConfigCommand::Show) | None => {
                    commands::config::handle().await.map_err(Into::into)
                }
                Some(cli::ConfigCommand::Set { key, value }) => {
                    commands::config::handle_set(&key, &value)
                        .await
                        .map_err(Into::into)
                }
                Some(cli::ConfigCommand::Get { key }) => {
                    commands::config::handle_get(&key).await.map_err(Into::into)
                }
                Some(cli::ConfigCommand::Reset { key }) => {
                    commands::config::handle_reset(key.clone())
                        .await
                        .map_err(Into::into)
                }
                Some(cli::ConfigCommand::Edit) => {
                    commands::config::handle_edit().await.map_err(Into::into)
                }
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
                    .map_err(Into::into)
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
                .map_err(Into::into)
            }
            Commands::Stats => commands::stats::handle(&self.registry)
                .await
                .map_err(Into::into),
            Commands::Plugin { command } => commands::plugin::handle(&self.registry, command)
                .await
                .map_err(Into::into),
            Commands::Venv { command } => commands::venv_cmd::handle(command)
                .await
                .map_err(Into::into),
            Commands::Global { command } => {
                commands::global::handle(command).await.map_err(Into::into)
            }
            Commands::Search {
                query,
                category,
                installed_only,
                available_only,
                format,
                verbose,
            } => commands::search::handle(
                &self.registry,
                query,
                category,
                installed_only,
                available_only,
                format,
                verbose,
            )
            .await
            .map_err(Into::into),
            Commands::Sync {
                check,
                force,
                dry_run,
                verbose,
                no_parallel,
                no_auto_install,
            } => commands::sync::handle(
                &self.registry,
                check,
                force,
                dry_run,
                verbose,
                no_parallel,
                no_auto_install,
            )
            .await
            .map_err(Into::into),

            Commands::Shell { command } => {
                use crate::cli::ShellCommand;
                match command {
                    ShellCommand::Init { shell } => {
                        commands::shell::handle_shell_init(shell.clone())
                            .await
                            .map_err(Into::into)
                    }
                    ShellCommand::Completions { shell } => {
                        commands::shell::handle_completion(shell.clone())
                            .await
                            .map_err(Into::into)
                    }
                }
            }
        }
    }

    /// Execute a tool with the given arguments
    async fn execute_tool(&self, args: &[String], use_system_path: bool) -> Result<()> {
        if args.is_empty() {
            return Err(VxError::Other {
                message: "No tool specified".to_string(),
            }
            .into());
        }

        let tool_name = &args[0];
        let tool_args = &args[1..];

        commands::execute::handle(&self.registry, tool_name, tool_args, use_system_path)
            .await
            .map_err(Into::into)
    }
}
