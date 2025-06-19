//! CLI command implementations

use crate::cli::{Cli, Commands};
use crate::ui::UI;
use vx_plugin::PluginRegistry;

pub mod cleanup;
pub mod config;
pub mod execute;
#[cfg(test)]
mod execute_tests;
pub mod fetch;
pub mod global;
pub mod init;
pub mod install;
pub mod list;
pub mod plugin;
pub mod remove;
pub mod search;
pub mod self_update;
pub mod shell;
pub mod stats;
pub mod switch;
pub mod sync;

pub mod update;
pub mod venv_cmd;
pub mod version;
pub mod where_cmd;

// Tests moved to tests/ directory

pub struct CommandHandler;

impl CommandHandler {
    pub async fn handle(cli: Cli, registry: &PluginRegistry) -> anyhow::Result<()> {
        // Set verbose mode
        UI::set_verbose(cli.verbose);

        match cli.command {
            Some(Commands::Version) => version::handle().await,

            Some(Commands::List {
                tool,
                status,
                installed: _,
                available: _,
            }) => list::handle(registry, tool.as_deref(), status).await,

            Some(Commands::Install {
                tool,
                version,
                force,
            }) => install::handle(registry, &tool, version.as_deref(), force).await,

            Some(Commands::Update { tool, apply: _ }) => {
                update::handle(registry, tool.as_deref(), false).await
            }

            Some(Commands::SelfUpdate {
                check,
                version: _,
                token,
                prerelease,
                force,
            }) => self_update::handle(token.as_deref(), prerelease, force, check).await,

            Some(Commands::Uninstall {
                tool,
                version,
                force,
            }) => remove::handle(registry, &tool, version.as_deref(), force).await,

            Some(Commands::Which { tool, all }) => where_cmd::handle(registry, &tool, all).await,

            Some(Commands::Versions {
                tool,
                latest,
                prerelease,
                detailed,
                interactive,
            }) => fetch::handle(registry, &tool, latest, detailed, interactive, prerelease).await,

            Some(Commands::Switch {
                tool_version,
                global,
            }) => switch::handle(registry, &tool_version, global).await,

            Some(Commands::Config { command }) => match command {
                Some(crate::cli::ConfigCommand::Show) | None => config::handle().await,
                Some(crate::cli::ConfigCommand::Set { key, value }) => {
                    config::handle_set(&key, &value).await
                }
                Some(crate::cli::ConfigCommand::Get { key }) => config::handle_get(&key).await,
                Some(crate::cli::ConfigCommand::Reset { key }) => {
                    config::handle_reset(key.clone()).await
                }
                Some(crate::cli::ConfigCommand::Edit) => config::handle_edit().await,
            },

            Some(Commands::Search {
                query,
                category,
                installed_only,
                available_only,
                format,
                verbose,
            }) => {
                // TODO: Get registry from context
                // For now, create a minimal registry
                let registry = PluginRegistry::new();
                search::handle(
                    &registry,
                    query.clone(),
                    category.clone(),
                    installed_only,
                    available_only,
                    format.clone(),
                    verbose,
                )
                .await
            }

            Some(Commands::Sync {
                check,
                force,
                dry_run,
                verbose,
                no_parallel,
                no_auto_install,
            }) => {
                // TODO: Get registry from context
                let registry = PluginRegistry::new();
                sync::handle(
                    &registry,
                    check,
                    force,
                    dry_run,
                    verbose,
                    no_parallel,
                    no_auto_install,
                )
                .await
            }

            Some(Commands::Init {
                interactive,
                template,
                tools,
                force,
                dry_run,
                list_templates,
            }) => {
                init::handle(
                    interactive,
                    template.clone(),
                    tools.clone(),
                    force,
                    dry_run,
                    list_templates,
                )
                .await
            }

            Some(Commands::Clean {
                dry_run,
                cache,
                orphaned,
                all,
                force,
                older_than,
                verbose,
            }) => {
                // Map new clean options to cleanup options
                let cache_only = cache && !all;
                let orphaned_only = orphaned && !all;
                cleanup::handle(
                    dry_run,
                    cache_only,
                    orphaned_only,
                    force,
                    older_than,
                    verbose,
                )
                .await
            }

            Some(Commands::Stats) => stats::handle(registry).await,

            Some(Commands::Plugin { command }) => plugin::handle(registry, command).await,

            Some(Commands::Venv { command }) => venv_cmd::handle(command).await,

            Some(Commands::Global { command }) => global::handle(command).await,

            None => {
                // Handle tool execution
                if cli.args.is_empty() {
                    UI::error("No tool specified");
                    UI::hint("Usage: vx <tool> [args...]");
                    UI::hint("Example: vx uv pip install requests");
                    UI::hint("Run 'vx list --all' to see supported tools");
                    std::process::exit(1);
                }

                let tool_name = &cli.args[0];
                let tool_args = &cli.args[1..];

                // Use the executor to run the tool
                let exit_code =
                    execute::execute_tool(registry, tool_name, tool_args, cli.use_system_path)
                        .await?;
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
                Ok(())
            }

            Some(Commands::Shell { command }) => {
                use crate::cli::ShellCommand;
                match command {
                    ShellCommand::Init { shell } => shell::handle_shell_init(shell.clone()).await,
                    ShellCommand::Completions { shell } => {
                        shell::handle_completion(shell.clone()).await
                    }
                }
            }
        }
    }
}
