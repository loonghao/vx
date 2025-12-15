//! Search command implementation

use crate::cli::OutputFormat;
use crate::ui::UI;
use anyhow::Result;
use vx_runtime::ProviderRegistry;

pub async fn handle(
    registry: &ProviderRegistry,
    query: Option<String>,
    _category: Option<String>,
    _installed_only: bool,
    _available_only: bool,
    _format: OutputFormat,
    _verbose: bool,
) -> Result<()> {
    let query = query.unwrap_or_default();
    let query_lower = query.to_lowercase();

    UI::header(&format!("Searching for '{}'...", query));

    let mut found = false;

    for name in registry.runtime_names() {
        if query.is_empty() || name.to_lowercase().contains(&query_lower) {
            if let Some(runtime) = registry.get_runtime(&name) {
                let description = runtime.description();
                if query.is_empty()
                    || description.to_lowercase().contains(&query_lower)
                    || name.to_lowercase().contains(&query_lower)
                {
                    UI::item(&format!("{} - {}", name, description));
                    found = true;
                }
            }
        }
    }

    if !found {
        UI::info(&format!("No runtimes found matching '{}'", query));
    }

    Ok(())
}
