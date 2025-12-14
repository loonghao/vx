//! Search command implementation

use crate::cli::OutputFormat;
use crate::ui::UI;
use anyhow::Result;
use vx_plugin::BundleRegistry;

pub async fn handle(
    _registry: &BundleRegistry,
    query: Option<String>,
    category: Option<String>,
    installed_only: bool,
    available_only: bool,
    format: OutputFormat,
    verbose: bool,
) -> Result<()> {
    UI::warning("Search command not yet fully implemented in new architecture");
    UI::hint(&format!(
        "Would search with query: {:?}, category: {:?}, installed_only: {}, available_only: {}, format: {:?}, verbose: {}",
        query, category, installed_only, available_only, format, verbose
    ));
    Ok(())
}
