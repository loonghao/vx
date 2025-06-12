//! Where command implementation

use crate::ui::UI;
use vx_core::{PluginRegistry, Result};

pub async fn handle(registry: &PluginRegistry, tool: &str, all: bool) -> Result<()> {
    UI::warn(&format!("Where command not yet implemented for: {}", tool));
    UI::hint("This will be implemented in the next iteration");
    Ok(())
}
