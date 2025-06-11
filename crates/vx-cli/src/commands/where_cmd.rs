//! Where command implementation

use vx_core::{PluginRegistry, Result};
use crate::ui::UI;

pub async fn handle(_registry: &PluginRegistry, tool: &str, _all: bool) -> Result<()> {
    UI::warn(&format!("Where command not yet implemented for: {}", tool));
    UI::hint("This will be implemented in the next iteration");
    Ok(())
}
