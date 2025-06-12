//! List command implementation

use crate::ui::UI;
use vx_core::{PluginRegistry, Result};

/// Handle the list command
pub async fn handle(registry: &PluginRegistry, show_all: bool) -> Result<()> {
    UI::info("Available tools:");

    let tools = registry.get_all_tools();

    if tools.is_empty() {
        UI::warn("No tools available");
        UI::hint("Make sure plugins are properly registered");
        return Ok(());
    }

    for tool in tools {
        // For now, show all tools without status checking
        UI::item(&format!("📦 {} - {}", tool.name(), tool.description()));

        // Show aliases if any
        let aliases = tool.aliases();
        if !aliases.is_empty() {
            UI::detail(&format!("   Aliases: {}", aliases.join(", ")));
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
