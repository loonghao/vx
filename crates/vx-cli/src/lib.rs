//! VX CLI - Command Line Interface for VX Tool Manager

use anyhow::Result;
use clap::Parser;
use vx_runtime::ProviderRegistry;

pub mod cli;
pub mod commands;
pub mod registry;
pub mod suggestions;
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
    async fn handle_command(&self, command: cli::Commands, cli: &Cli) -> Result<()> {
        use cli::Commands;

        match command {
            Commands::Version => commands::version::handle().await,
            Commands::List {
                tool,
                status,
                installed: _,
                available: _,
                all,
            } => {
                commands::list::handle(&self.registry, &self.context, tool.as_deref(), status, all)
                    .await
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
                Some(cli::ConfigCommand::Migrate {
                    path,
                    dry_run,
                    backup,
                    force,
                }) => commands::config::handle_migrate(path, dry_run, backup, force).await,
                Some(cli::ConfigCommand::Validate { path, verbose }) => {
                    commands::config::handle_validate(path, verbose).await
                }
                Some(cli::ConfigCommand::Schema { output }) => {
                    commands::config::handle_schema(output).await
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

            Commands::Dev {
                shell,
                command,
                no_install,
                verbose,
                export,
                format,
            } => commands::dev::handle(shell, command, no_install, verbose, export, format).await,

            Commands::Setup {
                force,
                dry_run,
                verbose,
                no_parallel,
                no_hooks,
            } => {
                commands::setup::handle(
                    &self.registry,
                    force,
                    dry_run,
                    verbose,
                    no_parallel,
                    no_hooks,
                )
                .await
            }

            Commands::Add { tool, version } => {
                commands::setup::add_tool(&tool, version.as_deref()).await
            }

            Commands::RemoveTool { tool } => commands::setup::remove_tool(&tool).await,

            Commands::Run { script, args } => self.run_script(&script, &args).await,

            Commands::Services { command } => {
                use crate::cli::ServicesCommand;
                match command {
                    ServicesCommand::Start {
                        services,
                        foreground,
                        force,
                        verbose,
                    } => {
                        let services = if services.is_empty() {
                            None
                        } else {
                            Some(services)
                        };
                        commands::services::handle_start(services, !foreground, force, verbose)
                            .await
                    }
                    ServicesCommand::Stop { services, verbose } => {
                        let services = if services.is_empty() {
                            None
                        } else {
                            Some(services)
                        };
                        commands::services::handle_stop(services, verbose).await
                    }
                    ServicesCommand::Status { verbose } => {
                        commands::services::handle_status(verbose).await
                    }
                    ServicesCommand::Logs {
                        service,
                        follow,
                        tail,
                    } => commands::services::handle_logs(&service, follow, tail).await,
                    ServicesCommand::Restart { services, verbose } => {
                        let services = if services.is_empty() {
                            None
                        } else {
                            Some(services)
                        };
                        commands::services::handle_restart(services, verbose).await
                    }
                }
            }

            Commands::Hook { command } => {
                use crate::cli::HookCommand;
                match command {
                    HookCommand::PreCommit => commands::hook::handle_pre_commit().await,
                    HookCommand::Enter => commands::hook::handle_enter().await,
                    HookCommand::Install { force } => commands::hook::handle_install(force).await,
                    HookCommand::Uninstall => commands::hook::handle_uninstall().await,
                    HookCommand::Status => commands::hook::handle_status().await,
                    HookCommand::Run { name } => commands::hook::handle_run(&name).await,
                    HookCommand::ShellInit { shell } => {
                        commands::hook::handle_shell_init(shell).await
                    }
                }
            }

            Commands::Container { command } => {
                use crate::cli::ContainerCommand;
                match command {
                    ContainerCommand::Generate {
                        output,
                        with_ignore,
                        dry_run,
                        template,
                    } => {
                        commands::container::handle_generate(output, with_ignore, dry_run, template)
                            .await
                    }
                    ContainerCommand::Build {
                        tag,
                        target,
                        build_arg,
                        platform,
                        no_cache,
                        push,
                        verbose,
                    } => {
                        commands::container::handle_build(
                            tag, target, build_arg, platform, no_cache, push, verbose,
                        )
                        .await
                    }
                    ContainerCommand::Push { tag, verbose } => {
                        commands::container::handle_push(tag, verbose).await
                    }
                    ContainerCommand::Status => commands::container::handle_status().await,
                    ContainerCommand::Login {
                        registry,
                        username,
                        password,
                    } => commands::container::handle_login(registry, username, password).await,
                    ContainerCommand::Tags { all } => commands::container::handle_tags(all).await,
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

    /// Run a script defined in .vx.toml
    async fn run_script(&self, script_name: &str, args: &[String]) -> Result<()> {
        use std::process::Command;

        let current_dir = std::env::current_dir()?;
        let config_path = current_dir.join(".vx.toml");

        if !config_path.exists() {
            return Err(anyhow::anyhow!("No .vx.toml found. Run 'vx init' first."));
        }

        let config = commands::setup::parse_vx_config(&config_path)?;

        let script_cmd = config.scripts.get(script_name).ok_or_else(|| {
            let available: Vec<_> = config.scripts.keys().collect();
            if available.is_empty() {
                anyhow::anyhow!("No scripts defined in .vx.toml")
            } else {
                anyhow::anyhow!(
                    "Script '{}' not found. Available scripts: {}",
                    script_name,
                    available
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        })?;

        ui::UI::info(&format!("Running script '{}': {}", script_name, script_cmd));

        // Parse the command
        let shell = if cfg!(windows) { "cmd" } else { "sh" };
        let shell_arg = if cfg!(windows) { "/C" } else { "-c" };

        // Append additional args to the command
        let full_cmd = if args.is_empty() {
            script_cmd.clone()
        } else {
            format!("{} {}", script_cmd, args.join(" "))
        };

        let status = Command::new(shell).arg(shell_arg).arg(&full_cmd).status()?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}
