//! Where command implementation

use crate::ui::UI;
use vx_core::{PluginRegistry, Result, VxEnvironment, VxError};

pub async fn handle(registry: &PluginRegistry, tool: &str, all: bool) -> Result<()> {
    // Get the tool from registry
    let vx_tool = registry
        .get_tool(tool)
        .ok_or_else(|| VxError::ToolNotFound {
            tool_name: tool.to_string(),
        })?;

    // Create environment manager
    let env = VxEnvironment::new().map_err(|e| VxError::Other {
        message: format!("Failed to create VX environment: {}", e),
    })?;

    // Get installed versions
    let installed_versions = vx_tool.get_installed_versions().await.unwrap_or_default();

    if installed_versions.is_empty() {
        UI::warn(&format!("Tool '{}' is not installed", tool));
        UI::hint(&format!("Use 'vx install {}' to install it", tool));
        UI::hint("Run 'vx list' to see available tools");
        return Ok(());
    }

    // Get active version
    let active_version = vx_tool.get_active_version().await.ok();

    UI::info(&format!("Tool: {}", tool));
    UI::detail(&format!("Description: {}", vx_tool.description()));

    if let Some(active_version) = &active_version {
        UI::success(&format!("Active version: {}", active_version));

        // Show active version details
        if let Ok(Some(installation)) = env.get_installation_info(tool, active_version) {
            UI::detail(&format!(
                "Install directory: {}",
                installation.install_dir.display()
            ));
            UI::detail(&format!(
                "Executable path: {}",
                installation.executable_path.display()
            ));
            UI::detail(&format!(
                "Installed at: {}",
                installation.installed_at.format("%Y-%m-%d %H:%M:%S UTC")
            ));
        }
    } else {
        UI::warn("No active version set");
    }

    // Show all versions if requested or if there are multiple versions
    if all || installed_versions.len() > 1 {
        UI::info(&format!(
            "\nAll installed versions ({}):",
            installed_versions.len()
        ));

        for version in &installed_versions {
            let is_active = active_version.as_ref() == Some(version);
            let status_icon = if is_active { "✅" } else { "📦" };

            UI::item(&format!("{} {}", status_icon, version));

            // Show installation details for each version
            if let Ok(Some(installation)) = env.get_installation_info(tool, version) {
                UI::detail(&format!("   Path: {}", installation.install_dir.display()));
                if is_active {
                    UI::detail(&format!(
                        "   Executable: {}",
                        installation.executable_path.display()
                    ));
                }
            }
        }

        if !all && installed_versions.len() > 1 {
            UI::hint("Use --all to see details for all versions");
        }
    }

    Ok(())
}
