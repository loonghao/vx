// Main command dispatcher
// Routes commands to their respective handlers

use crate::cli::{Cli, Commands};
use crate::ui::UI;
use anyhow::Result;

pub struct CommandHandler;

impl CommandHandler {
    pub async fn handle(cli: Cli) -> Result<()> {
        // Set verbose mode
        UI::set_verbose(cli.verbose);

        match cli.command {
            Some(Commands::Version) => super::version::handle().await,

            Some(Commands::List) => super::list::handle(None, false, false).await,

            Some(Commands::Install {
                tool,
                version,
                force,
            }) => super::install::handle(tool, version, force).await,

            Some(Commands::Update { tool, apply: _ }) => {
                super::update::handle(tool, false, None).await
            }

            Some(Commands::Remove {
                tool,
                version,
                force,
            }) => super::remove::handle(tool, version, force).await,

            Some(Commands::Where { tool, all }) => super::where_cmd::handle(tool, all).await,

            Some(Commands::Fetch {
                tool,
                latest,
                prerelease,
                detailed,
                interactive,
            }) => super::fetch::handle(tool, latest, prerelease, detailed, interactive).await,

            Some(Commands::Use { tool_version }) => super::use_cmd::handle(tool_version).await,

            Some(Commands::Switch {
                tool_version,
                global,
            }) => super::switch::handle(tool_version, global).await,

            Some(Commands::Config) => super::config::handle(None).await,

            Some(Commands::Init) => super::config::handle_init(vec![], None).await,

            Some(Commands::Cleanup) => super::stats::handle_cleanup(false, false, false).await,

            Some(Commands::Stats) => super::stats::handle(None, false).await,

            Some(Commands::Plugin { command }) => super::plugin::handle(command).await,

            Some(Commands::Venv { command }) => {
                super::venv_cmd::handle_venv_command(crate::cli::venv_cmd::VenvArgs { command })
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
                    crate::executor::execute_tool(tool_name, tool_args, cli.use_system_path)
                        .await?;
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
                Ok(())
            }
        }
    }
}
