//! Switch command implementation

use crate::ui::UI;
use vx_core::{PluginRegistry, Result};

pub async fn handle(registry: &PluginRegistry, tool_version: &str, global: bool) -> Result<()> {
    UI::warn(&format!(
        "Switch command not yet implemented for: {}",
        tool_version
    ));
    UI::hint("This will be implemented in the next iteration");
    Ok(())
}
