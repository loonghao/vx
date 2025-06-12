// Use command implementation (deprecated, redirects to switch)

use crate::ui::UI;
use vx_core::{PluginRegistry, Result};

pub async fn handle(registry: &PluginRegistry, tool_version: &str) -> Result<()> {
    UI::warn("'use' command is deprecated, use 'switch' instead");
    UI::hint(&format!("Try: vx switch {}", tool_version));

    // Redirect to switch command
    super::switch::handle(registry, tool_version, false).await
}
