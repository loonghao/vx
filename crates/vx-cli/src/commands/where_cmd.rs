//! Where command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_plugin::PluginRegistry;

pub async fn handle(_registry: &PluginRegistry, tool: &str, all: bool) -> Result<()> {
    UI::warning("Where command not yet fully implemented in new architecture");
    UI::hint(&format!(
        "Would show location of tool: {} (all: {})",
        tool, all
    ));
    Ok(())
}
