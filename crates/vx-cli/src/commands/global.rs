//! Global tool management commands

use crate::ui::UI;
use clap::Subcommand;
use vx_core::{GlobalToolManager, Result};

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
    let global_manager = GlobalToolManager::new()?;

    match command {
        GlobalCommand::List { verbose } => {
            let tools = global_manager.list_global_tools().await?;

            if tools.is_empty() {
                UI::info("No global tools installed");
                UI::hint(
                    "Install tools with 'vx install <tool>' or run 'vx <tool>' to auto-install",
                );
                return Ok(());
            }

            UI::info(&format!("Global tools ({} installed):", tools.len()));

            for tool in tools {
                if verbose {
                    UI::detail(&format!("📦 {} v{}", tool.name, tool.version));
                    UI::detail(&format!("   Path: {}", tool.install_path.display()));
                    UI::detail(&format!(
                        "   Installed: {}",
                        tool.installed_at.format("%Y-%m-%d %H:%M:%S")
                    ));

                    if !tool.referenced_by.is_empty() {
                        let refs: Vec<String> = tool.referenced_by.iter().cloned().collect();
                        UI::detail(&format!("   Referenced by: {}", refs.join(", ")));
                    } else {
                        UI::detail("   Referenced by: none");
                    }
                    println!();
                } else {
                    let refs = if tool.referenced_by.is_empty() {
                        "".to_string()
                    } else {
                        format!(" (used by {})", tool.referenced_by.len())
                    };
                    UI::detail(&format!("📦 {} v{}{}", tool.name, tool.version, refs));
                }
            }
        }

        GlobalCommand::Info { tool_name } => {
            if let Some(tool) = global_manager.get_tool_info(&tool_name).await? {
                UI::info(&format!("Global tool: {}", tool.name));
                UI::detail(&format!("Version: {}", tool.version));
                UI::detail(&format!("Install path: {}", tool.install_path.display()));
                UI::detail(&format!(
                    "Installed at: {}",
                    tool.installed_at.format("%Y-%m-%d %H:%M:%S")
                ));

                if tool.referenced_by.is_empty() {
                    UI::detail("Referenced by: none (can be safely removed)");
                } else {
                    UI::detail("Referenced by virtual environments:");
                    for venv in &tool.referenced_by {
                        UI::detail(&format!("  - {}", venv));
                    }
                }
            } else {
                UI::error(&format!("Global tool '{}' not found", tool_name));
                UI::hint("Run 'vx global list' to see all installed global tools");
            }
        }

        GlobalCommand::Remove { tool_name, force } => {
            if !global_manager.is_tool_installed(&tool_name).await? {
                UI::error(&format!("Global tool '{}' is not installed", tool_name));
                return Ok(());
            }

            if !force && !global_manager.can_remove_tool(&tool_name).await? {
                let dependents = global_manager.get_tool_dependents(&tool_name).await?;
                UI::error(&format!(
                    "Cannot remove tool '{}' - it is referenced by virtual environments:",
                    tool_name
                ));
                for venv in dependents {
                    UI::detail(&format!("  - {}", venv));
                }
                UI::hint("Use --force to remove anyway, or remove the tool from virtual environments first");
                return Ok(());
            }

            if force {
                UI::warn(&format!("Force removing global tool '{}'...", tool_name));
            } else {
                UI::info(&format!("Removing global tool '{}'...", tool_name));
            }

            global_manager.remove_global_tool(&tool_name).await?;
            UI::success(&format!("Successfully removed global tool '{}'", tool_name));
        }

        GlobalCommand::Dependents { tool_name } => {
            let dependents = global_manager.get_tool_dependents(&tool_name).await?;

            if dependents.is_empty() {
                UI::info(&format!(
                    "Tool '{}' is not referenced by any virtual environments",
                    tool_name
                ));
                UI::hint("This tool can be safely removed");
            } else {
                UI::info(&format!(
                    "Tool '{}' is referenced by {} virtual environment(s):",
                    tool_name,
                    dependents.len()
                ));
                for venv in dependents {
                    UI::detail(&format!("  - {}", venv));
                }
            }
        }

        GlobalCommand::Cleanup { dry_run } => {
            let tools = global_manager.list_global_tools().await?;
            let mut removable_tools = Vec::new();

            for tool in tools {
                if tool.referenced_by.is_empty() {
                    removable_tools.push(tool);
                }
            }

            if removable_tools.is_empty() {
                UI::info("No unused global tools found");
                return Ok(());
            }

            if dry_run {
                UI::info(&format!(
                    "Would remove {} unused global tool(s):",
                    removable_tools.len()
                ));
                for tool in removable_tools {
                    UI::detail(&format!("  - {} v{}", tool.name, tool.version));
                }
                UI::hint("Run without --dry-run to actually remove these tools");
            } else {
                UI::info(&format!(
                    "Removing {} unused global tool(s)...",
                    removable_tools.len()
                ));

                let mut removed_count = 0;
                for tool in removable_tools {
                    match global_manager.remove_global_tool(&tool.name).await {
                        Ok(()) => {
                            UI::detail(&format!("✓ Removed {} v{}", tool.name, tool.version));
                            removed_count += 1;
                        }
                        Err(e) => {
                            UI::error(&format!("✗ Failed to remove {}: {}", tool.name, e));
                        }
                    }
                }

                UI::success(&format!(
                    "Successfully removed {} global tool(s)",
                    removed_count
                ));
            }
        }
    }

    Ok(())
}
