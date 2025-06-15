//! Symlink virtual environment command implementation

use crate::ui::UI;
use clap::Subcommand;
use std::path::PathBuf;
use vx_core::{Result, SymlinkVenvManager};

#[derive(Subcommand, Clone)]
pub enum SymlinkVenvCommand {
    /// Create a new symlink virtual environment
    Create {
        /// Virtual environment name
        name: String,
        /// Custom path for the virtual environment
        #[arg(short, long)]
        path: Option<String>,
    },
    /// List all symlink virtual environments
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Remove a symlink virtual environment
    Remove {
        /// Virtual environment name
        name: String,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Show symlink virtual environment information
    Info {
        /// Virtual environment name
        name: String,
    },
    /// Link a global tool to a virtual environment
    Link {
        /// Virtual environment name
        venv_name: String,
        /// Tool name to link
        tool_name: String,
        /// Tool version to link (defaults to latest)
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Unlink a tool from a virtual environment
    Unlink {
        /// Virtual environment name
        venv_name: String,
        /// Tool name to unlink
        tool_name: String,
    },
}

/// Handle symlink virtual environment commands
pub async fn handle(command: SymlinkVenvCommand) -> Result<()> {
    let venv_manager = SymlinkVenvManager::new()?;

    match command {
        SymlinkVenvCommand::Create { name, path } => {
            let venv_path = if let Some(custom_path) = path {
                PathBuf::from(custom_path)
            } else {
                std::env::current_dir()?.join(&name)
            };

            UI::info(&format!(
                "Creating symlink virtual environment '{}'...",
                name
            ));
            UI::detail(&format!("Path: {}", venv_path.display()));

            venv_manager.create_venv(&name, &venv_path).await?;

            UI::success(&format!(
                "Symlink virtual environment '{}' created successfully",
                name
            ));
            UI::hint(&format!(
                "Link tools with: vx symlink-venv link {} <tool>",
                name
            ));
            UI::hint(&format!(
                "Add to PATH: export PATH=\"{}:$PATH\"",
                venv_path.join("bin").display()
            ));
        }

        SymlinkVenvCommand::List { verbose } => {
            let venvs = venv_manager.list_venvs().await?;

            if venvs.is_empty() {
                UI::info("No symlink virtual environments found");
                UI::hint("Create one with: vx symlink-venv create <name>");
                return Ok(());
            }

            UI::info(&format!(
                "Symlink virtual environments ({} found):",
                venvs.len()
            ));

            for venv in venvs {
                if verbose {
                    UI::detail(&format!("🔗 {} ({})", venv.name, venv.path.display()));
                    UI::detail(&format!(
                        "   Created: {}",
                        venv.created_at.format("%Y-%m-%d %H:%M:%S")
                    ));
                    UI::detail(&format!(
                        "   Modified: {}",
                        venv.modified_at.format("%Y-%m-%d %H:%M:%S")
                    ));

                    if venv.linked_tools.is_empty() {
                        UI::detail("   Linked tools: none");
                    } else {
                        UI::detail("   Linked tools:");
                        for (tool, version) in &venv.linked_tools {
                            UI::detail(&format!("     - {} v{}", tool, version));
                        }
                    }
                    println!();
                } else {
                    let tools_count = venv.linked_tools.len();
                    let tools_info = if tools_count == 0 {
                        "no tools".to_string()
                    } else {
                        format!("{} tool(s)", tools_count)
                    };
                    UI::detail(&format!(
                        "🔗 {} ({}) - {}",
                        venv.name,
                        venv.path.display(),
                        tools_info
                    ));
                }
            }
        }

        SymlinkVenvCommand::Remove { name, force } => {
            if !venv_manager.venv_exists(&name).await? {
                UI::error(&format!("Symlink virtual environment '{}' not found", name));
                return Ok(());
            }

            if !force {
                UI::warn(&format!(
                    "This will permanently remove symlink virtual environment '{}'",
                    name
                ));
                UI::hint("Use --force to skip this confirmation");
                return Ok(());
            }

            UI::info(&format!(
                "Removing symlink virtual environment '{}'...",
                name
            ));
            venv_manager.remove_venv(&name).await?;
            UI::success(&format!("Symlink virtual environment '{}' removed", name));
        }

        SymlinkVenvCommand::Info { name } => {
            if let Some(venv) = venv_manager.get_venv(&name).await? {
                UI::info(&format!("Symlink virtual environment: {}", venv.name));
                UI::detail(&format!("Path: {}", venv.path.display()));
                UI::detail(&format!(
                    "Bin directory: {}",
                    venv.path.join("bin").display()
                ));
                UI::detail(&format!(
                    "Created: {}",
                    venv.created_at.format("%Y-%m-%d %H:%M:%S")
                ));
                UI::detail(&format!(
                    "Modified: {}",
                    venv.modified_at.format("%Y-%m-%d %H:%M:%S")
                ));

                if venv.linked_tools.is_empty() {
                    UI::detail("Linked tools: none");
                    UI::hint(&format!(
                        "Link tools with: vx symlink-venv link {} <tool>",
                        name
                    ));
                } else {
                    UI::detail(&format!("Linked tools ({}):", venv.linked_tools.len()));
                    for (tool, version) in &venv.linked_tools {
                        UI::detail(&format!("  - {} v{}", tool, version));
                    }
                }

                UI::hint(&format!(
                    "Add to PATH: export PATH=\"{}:$PATH\"",
                    venv.path.join("bin").display()
                ));
            } else {
                UI::error(&format!("Symlink virtual environment '{}' not found", name));
                UI::hint("Run 'vx symlink-venv list' to see all symlink virtual environments");
            }
        }

        SymlinkVenvCommand::Link {
            venv_name,
            tool_name,
            version,
        } => {
            let version = version.unwrap_or_else(|| "latest".to_string());

            UI::info(&format!(
                "Linking {} v{} to symlink virtual environment '{}'...",
                tool_name, version, venv_name
            ));

            venv_manager
                .link_tool(&venv_name, &tool_name, &version)
                .await?;

            UI::success(&format!(
                "Successfully linked {} v{} to '{}'",
                tool_name, version, venv_name
            ));

            if let Some(venv) = venv_manager.get_venv(&venv_name).await? {
                UI::hint(&format!(
                    "Tool is now available at: {}",
                    venv.path.join("bin").join(&tool_name).display()
                ));
            }
        }

        SymlinkVenvCommand::Unlink {
            venv_name,
            tool_name,
        } => {
            UI::info(&format!(
                "Unlinking {} from symlink virtual environment '{}'...",
                tool_name, venv_name
            ));

            venv_manager.unlink_tool(&venv_name, &tool_name).await?;

            UI::success(&format!(
                "Successfully unlinked {} from '{}'",
                tool_name, venv_name
            ));
        }
    }

    Ok(())
}
