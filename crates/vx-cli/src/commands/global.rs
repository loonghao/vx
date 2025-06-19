//! Global tool management commands

use crate::ui::UI;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand, Clone)]
pub enum GlobalCommand {
    /// List all globally installed tools
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show information about a specific global tool
    Info {
        /// Tool name
        tool_name: String,
    },
    /// Remove a global tool (only if not referenced by any venv)
    Remove {
        /// Tool name
        tool_name: String,
        /// Force removal even if referenced by virtual environments
        #[arg(short, long)]
        force: bool,
    },
    /// Show which virtual environments depend on a tool
    Dependents {
        /// Tool name
        tool_name: String,
    },
    /// Clean up unused global tools
    Cleanup {
        /// Dry run - show what would be removed without actually removing
        #[arg(short, long)]
        dry_run: bool,
    },
}

/// Handle global tool management commands
pub async fn handle(command: GlobalCommand) -> Result<()> {
    UI::warning("Global commands not yet implemented in new architecture");

    match command {
        GlobalCommand::List { verbose: _ } => {
            UI::hint("Would list global tools");
        }

        GlobalCommand::Info { tool_name } => {
            UI::hint(&format!("Would show info for global tool: {}", tool_name));
        }

        GlobalCommand::Remove { tool_name, force } => {
            UI::hint(&format!(
                "Would remove global tool: {} (force: {})",
                tool_name, force
            ));
        }

        GlobalCommand::Dependents { tool_name } => {
            UI::hint(&format!(
                "Would show dependents for global tool: {}",
                tool_name
            ));
        }

        GlobalCommand::Cleanup { dry_run } => {
            UI::hint(&format!(
                "Would cleanup global tools (dry_run: {})",
                dry_run
            ));
        }
    }

    Ok(())
}
