// Use command implementation (deprecated, redirects to switch)

use crate::ui::UI;
use anyhow::Result;

pub async fn handle(tool_version: String) -> Result<()> {
    UI::warning("'use' command is deprecated, use 'switch' instead");
    UI::hint(&format!("Try: vx switch {}", tool_version));

    // Redirect to switch command
    crate::cli::switch::handle(tool_version, false).await
}
