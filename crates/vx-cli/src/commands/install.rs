//! Install command implementation

use vx_core::{PluginRegistry, Result};
use crate::ui::UI;

pub async fn handle(_registry: &PluginRegistry, tool: &str, _force: bool) -> Result<()> {
    UI::warn(&format!("Install command not yet implemented for: {}", tool));
    UI::hint("This will be implemented in the next iteration");
    Ok(())
}
