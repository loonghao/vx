//! Switch command implementation
//!
//! Switches the active version of a tool.
//!
//! ## Usage
//!
//! ```bash
//! vx switch node@20.10.0        # Switch to specific version
//! vx switch node@20.10.0 --global  # Switch globally
//! ```

use super::common::parse_tool_version;
use crate::ui::UI;
use anyhow::Result;
use vx_runtime::ProviderRegistry;

/// Handle the switch command
pub async fn handle(_registry: &ProviderRegistry, tool_version: &str, global: bool) -> Result<()> {
    // Parse tool@version format
    let (tool_name, version) = parse_tool_version(tool_version)?;

    UI::info(&format!("Switching {} to version {}", tool_name, version));

    // TODO: Implement version switching
    // 1. Check if version is installed
    // 2. Update default version symlink/config
    // 3. If global, update global config; otherwise update project config

    UI::warning("Switch command not yet fully implemented");
    UI::hint(&format!(
        "Would switch {} to version {} (global: {})",
        tool_name, version, global
    ));
    UI::hint("For now, use 'vx install <tool>@<version>' to install a specific version");

    Ok(())
}
