//! List command implementation

use crate::ui::UI;
use vx_core::{PluginRegistry, Result, VxEnvironment, VxError};

/// Handle the list command
pub async fn handle(
    registry: &PluginRegistry,
    tool_name: Option<&str>,
    show_status: bool,
) -> Result<()> {
    // If a specific tool is requested, show details for that tool
    if let Some(tool) = tool_name {
        return show_tool_details(registry, tool, show_status).await;
    }

    // Show all tools
    UI::info("Available tools:");

    let tools = registry.get_all_tools();

    if tools.is_empty() {
        UI::warn("No tools available");
        UI::hint("Make sure plugins are properly registered");
        return Ok(());
    }

    for tool in tools {
        let status_icon = if show_status {
            // Use the tool's own method to check installation status
            let installed_versions = tool.get_installed_versions().await.unwrap_or_default();
            if installed_versions.is_empty() {
                "❌"
            } else {
                "✅"
            }
        } else {
            "📦"
        };

        UI::item(&format!(
            "{} {} - {}",
            status_icon,
            tool.name(),
            tool.description()
        ));

        // Show aliases if any
        let aliases = tool.aliases();
        if !aliases.is_empty() {
            UI::detail(&format!("   Aliases: {}", aliases.join(", ")));
        }

        // Show installation status if requested
        if show_status {
            let installed_versions = tool.get_installed_versions().await.unwrap_or_default();
            if !installed_versions.is_empty() {
                UI::detail(&format!(
                    "   Installed versions: {}",
                    installed_versions.join(", ")
                ));

                // Show active version if any
                if let Ok(active_version) = tool.get_active_version().await {
                    UI::detail(&format!("   Active version: {}", active_version));
                }
            }
        }
    }

    // Show package managers
    UI::info("\nAvailable package managers:");
    let package_managers = registry.get_all_package_managers();

    if package_managers.is_empty() {
        UI::warn("No package managers available");
    } else {
        for pm in package_managers {
            let available = pm.is_available().await.unwrap_or(false);
            let status_icon = if available { "✅" } else { "❌" };

            UI::item(&format!(
                "{} {} ({:?}) - {}",
                status_icon,
                pm.name(),
                pm.ecosystem(),
                pm.description()
            ));
        }
    }

    Ok(())
}

/// Show detailed information for a specific tool
async fn show_tool_details(
    registry: &PluginRegistry,
    tool_name: &str,
    show_status: bool,
) -> Result<()> {
    let tool = registry
        .get_tool(tool_name)
        .ok_or_else(|| VxError::ToolNotFound {
            tool_name: tool_name.to_string(),
        })?;

    UI::header(&format!("Tool: {}", tool.name()));
    UI::info(&format!("Description: {}", tool.description()));

    // Show aliases if any
    let aliases = tool.aliases();
    if !aliases.is_empty() {
        UI::info(&format!("Aliases: {}", aliases.join(", ")));
    }

    // Show metadata
    let metadata = tool.metadata();
    if !metadata.is_empty() {
        UI::info("Metadata:");
        for (key, value) in metadata {
            UI::detail(&format!("  {}: {}", key, value));
        }
    }

    if show_status {
        // Create environment manager for status checking
        let env = VxEnvironment::new().map_err(|e| VxError::Other {
            message: format!("Failed to create VX environment: {}", e),
        })?;

        // Show installation status
        let installed_versions = tool.get_installed_versions().await.unwrap_or_default();

        if installed_versions.is_empty() {
            UI::warn("Not installed");
            UI::hint(&format!("Use 'vx install {}' to install it", tool.name()));
        } else {
            UI::success(&format!(
                "Installed versions ({}): {}",
                installed_versions.len(),
                installed_versions.join(", ")
            ));

            // Show active version if any
            if let Ok(active_version) = tool.get_active_version().await {
                UI::info(&format!("Active version: {}", active_version));

                // Show installation path
                if let Ok(Some(installation)) =
                    env.get_installation_info(tool.name(), &active_version)
                {
                    UI::detail(&format!(
                        "Installation path: {}",
                        installation.install_dir.display()
                    ));
                    UI::detail(&format!(
                        "Executable: {}",
                        installation.executable_path.display()
                    ));
                }
            } else {
                UI::warn("No active version set");
                UI::hint(&format!(
                    "Use 'vx switch {}@<version>' to set active version",
                    tool.name()
                ));
            }
        }

        // Show related tools (tools from the same plugin)
        if let Some(plugin) = registry.find_plugin_for_tool(tool.name()) {
            let related_tools: Vec<String> = plugin
                .tools()
                .into_iter()
                .filter(|t| t.name() != tool.name())
                .map(|t| t.name().to_string())
                .collect();

            if !related_tools.is_empty() {
                UI::info(&format!("Related tools: {}", related_tools.join(", ")));
            }
        }
    }

    Ok(())
}
