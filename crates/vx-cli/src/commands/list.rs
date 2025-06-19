//! List command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_plugin::PluginRegistry;

pub async fn handle(
    _registry: &PluginRegistry,
    tool: Option<&str>,
    show_status: bool,
) -> Result<()> {
    UI::warning("List command not yet fully implemented in new architecture");

    match tool {
        Some(tool_name) => {
            UI::hint(&format!(
                "Would list versions for tool: {} (show_status: {})",
                tool_name, show_status
            ));
        }
        None => {
            UI::hint(&format!(
                "Would list all tools (show_status: {})",
                show_status
            ));
        }
    }
    Ok(())
}
