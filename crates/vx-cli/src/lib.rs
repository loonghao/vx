//! VX CLI - Command Line Interface for VX Tool Manager

use anyhow::Result;
use clap::Parser;
use vx_plugin::PluginRegistry;

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
    let registry = PluginRegistry::new();

    // Register Node.js plugin
    registry
        .register_plugin(Box::new(vx_tool_node::NodePlugin::new()))
        .await?;

    // Register Go plugin
    registry
        .register_plugin(Box::new(vx_tool_go::GoPlugin::new()))
        .await?;

    // Register Rust plugin
    registry
        .register_plugin(Box::new(vx_tool_rust::RustPlugin::new()))
        .await?;

    // Register UV plugin
    registry
        .register_plugin(Box::new(vx_tool_uv::UvPlugin::new()))
        .await?;

    // Register Bun plugin
    registry
        .register_plugin(Box::new(vx_tool_bun::BunPlugin::new()))
        .await?;

    // Register Yarn plugin
    registry
        .register_plugin(Box::new(vx_tool_yarn::YarnPlugin::new()))
        .await?;

    // Register PNPM plugin
    registry
        .register_plugin(Box::new(vx_tool_pnpm::PnpmPlugin::new()))
        .await?;

    // Register Python plugin
    registry
        .register_plugin(vx_tool_python::create_plugin())
        .await?;

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
    async fn handle_command(&self, command: cli::Commands, cli: &Cli) -> Result<()> {
        use cli::Commands;

        match command {
            Commands::Version => commands::version::handle().await,
            Commands::List {
                tool,
                status,
                installed: _,
                available: _,
            } => commands::list::handle(&self.registry, tool.as_deref(), status).await,
            Commands::Install {
                tool,
                version,
                force,
            } => commands::install::handle(&self.registry, &tool, version.as_deref(), force).await,
            Commands::Update { tool, apply } => {
                commands::update::handle(&self.registry, tool.as_deref(), apply).await
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
            } => commands::remove::handle(&self.registry, &tool, version.as_deref(), force).await,

            Commands::Which { tool, all } => {
                commands::where_cmd::handle(&self.registry, &tool, all, cli.use_system_path).await
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
            Commands::Config { action } => {
                let config_cmd = commands::config::ConfigCommand { action };
                config_cmd.execute().await
            }
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
            Commands::Async { command } => {
                use crate::cli::AsyncCommand;
                match command {
                    AsyncCommand::Install {
                        tools,
                        force,
                        max_concurrent,
                    } => {
                        commands::async_install::handle_concurrent(&tools, force, max_concurrent)
                            .await
                    }
                    AsyncCommand::Versions { tools, prerelease } => {
                        commands::async_install::handle_versions_concurrent(&tools, prerelease)
                            .await
                    }
                    AsyncCommand::Benchmark { tools, force } => {
                        commands::async_install::handle_benchmark(&tools, force).await
                    }
                    AsyncCommand::Stats => commands::async_install::handle_stats().await,
                    AsyncCommand::ClearCache => commands::async_install::handle_clear_cache().await,
                }
            }
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

        commands::execute::handle(&self.registry, tool_name, tool_args, use_system_path).await
    }
}
