//! CLI command implementations

use crate::cli::{Cli, Commands};
use crate::ui::UI;
use vx_core::PluginRegistry;

pub mod config;
pub mod execute;
pub mod fetch;
pub mod install;
pub mod list;
pub mod plugin;
pub mod remove;
pub mod stats;
pub mod switch;
pub mod update;
pub mod use_cmd;
pub mod venv_cmd;
pub mod version;
pub mod where_cmd;

#[cfg(test)]
pub mod tests;

pub struct CommandHandler;

impl CommandHandler {
    pub async fn handle(cli: Cli, registry: &PluginRegistry) -> anyhow::Result<()> {
        // Set verbose mode
        UI::set_verbose(cli.verbose);

        match cli.command {
            Some(Commands::Version) => version::handle().await.map_err(Into::into),

            Some(Commands::List { tool, status }) => {
                list::handle(registry, tool.as_deref(), status)
                    .await
                    .map_err(Into::into)
            }

            Some(Commands::Install {
                tool,
                version,
                force,
            }) => install::handle(registry, &tool, version.as_deref(), force)
                .await
                .map_err(Into::into),

            Some(Commands::Update { tool, apply: _ }) => {
                update::handle(registry, tool.as_deref(), false)
                    .await
                    .map_err(Into::into)
            }

            Some(Commands::Remove {
                tool,
                version,
                force,
            }) => remove::handle(registry, &tool, version.as_deref(), force)
                .await
                .map_err(Into::into),

            Some(Commands::Where { tool, all }) => where_cmd::handle(registry, &tool, all)
                .await
                .map_err(Into::into),

            Some(Commands::Fetch {
                tool,
                latest,
                prerelease,
                detailed,
                interactive,
            }) => fetch::handle(registry, &tool, latest, detailed, interactive, prerelease)
                .await
                .map_err(Into::into),

            Some(Commands::Use { tool_version }) => use_cmd::handle(registry, &tool_version)
                .await
                .map_err(Into::into),

            Some(Commands::Switch {
                tool_version,
                global,
            }) => switch::handle(registry, &tool_version, global)
                .await
                .map_err(Into::into),

            Some(Commands::Config) => config::handle().await.map_err(Into::into),

            Some(Commands::Init) => config::handle_init(vec![], None).await.map_err(Into::into),

            Some(Commands::Cleanup) => stats::handle_cleanup(false, false, false)
                .await
                .map_err(Into::into),

            Some(Commands::Stats) => stats::handle(registry).await.map_err(Into::into),

            Some(Commands::Plugin { command }) => {
                plugin::handle(registry, command).await.map_err(Into::into)
            }

            Some(Commands::Venv { command }) => venv_cmd::handle(command).await.map_err(Into::into),

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
                        .await
                        .map_err(anyhow::Error::from)?;
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
                Ok(())
            }
        }
    }
}
