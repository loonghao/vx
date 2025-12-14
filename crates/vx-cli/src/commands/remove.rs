//! Remove command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_plugin::BundleRegistry;

pub async fn handle(
    registry: &BundleRegistry,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
) -> Result<()> {
    // Get the tool from registry
    let tool = registry
        .get_tool(tool_name)
        .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

    if let Some(target_version) = version {
        // Remove specific version
        UI::info(&format!("Removing {} {}...", tool_name, target_version));

        match tool.remove_version(target_version, force).await {
            Ok(()) => {
                UI::success(&format!(
                    "Successfully removed {} {}",
                    tool_name, target_version
                ));
            }
            Err(e) => {
                UI::error(&format!(
                    "Failed to remove {} {}: {}",
                    tool_name, target_version, e
                ));
                return Err(e);
            }
        }
    } else {
        // Remove all versions
        let installed_versions = tool.get_installed_versions().await?;

        if installed_versions.is_empty() {
            UI::warn(&format!("No versions of {} are installed", tool_name));
            return Ok(());
        }

        if !force {
            UI::warn(&format!(
                "This will remove all {} versions: {}",
                tool_name,
                installed_versions.join(", ")
            ));
            UI::hint("Use --force to confirm removal of all versions");
            return Ok(());
        }

        UI::info(&format!("Removing all {} versions...", tool_name));

        let mut errors = Vec::new();
        for version in &installed_versions {
            match tool.remove_version(version, true).await {
                Ok(()) => {
                    UI::detail(&format!("Removed {} {}", tool_name, version));
                }
                Err(e) => {
                    UI::error(&format!(
                        "Failed to remove {} {}: {}",
                        tool_name, version, e
                    ));
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            UI::success(&format!("Successfully removed all {} versions", tool_name));
        } else {
            UI::warn(&format!(
                "Removed some versions, but {} errors occurred",
                errors.len()
            ));
        }
    }

    Ok(())
}
