//! CLI command implementations

use crate::cli::{Cli, Commands};
use crate::ui::UI;
use anyhow::Result;

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

pub struct CommandHandler;

impl CommandHandler {
    pub async fn handle(cli: Cli) -> Result<()> {
        // Set verbose mode
        UI::set_verbose(cli.verbose);

        match cli.command {
            Some(Commands::Version) => version::handle().await,

            Some(Commands::List) => list::handle(None, false, false).await,

            Some(Commands::Install {
                tool,
                version,
                force,
            }) => install::handle(tool, version, force).await,

            Some(Commands::Update { tool, apply: _ }) => {
                update::handle(tool, false, None).await
            }

            Some(Commands::Remove {
                tool,
                version,
                force,
            }) => remove::handle(tool, version, force).await,

            Some(Commands::Where { tool, all }) => where_cmd::handle(tool, all).await,

            Some(Commands::Fetch {
                tool,
                latest,
                prerelease,
                detailed,
                interactive,
            }) => fetch::handle(tool, latest, prerelease, detailed, interactive).await,

            Some(Commands::Use { tool_version }) => use_cmd::handle(tool_version).await,

            Some(Commands::Switch {
                tool_version,
                global,
            }) => switch::handle(tool_version, global).await,

            Some(Commands::Config) => config::handle(None).await,

            Some(Commands::Init) => config::handle_init(vec![], None).await,

            Some(Commands::Cleanup) => stats::handle_cleanup(false, false, false).await,

            Some(Commands::Stats) => stats::handle(None, false).await,

            Some(Commands::Plugin { command }) => plugin::handle(command).await,

            Some(Commands::Venv { command }) => {
                venv_cmd::handle_venv_command(venv_cmd::VenvArgs { command })
                    .await
            }

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
                    execute::execute_tool(tool_name, tool_args, cli.use_system_path)
                        .await?;
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
                Ok(())
            }
        }
    }
}
