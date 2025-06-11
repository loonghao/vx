//! Switch command implementation

use vx_core::{PluginRegistry, Result};
use crate::ui::UI;

pub async fn handle(_registry: &PluginRegistry, tool_version: &str, _global: bool) -> Result<()> {
    UI::warn(&format!("Switch command not yet implemented for: {}", tool_version));
    UI::hint("This will be implemented in the next iteration");
    Ok(())
}
