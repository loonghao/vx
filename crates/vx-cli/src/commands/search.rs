//! Search command implementation (RFC 0031: unified structured output)

use crate::cli::OutputFormat;
use crate::output::{CommandOutput, OutputRenderer};
use crate::ui::UI;
use anyhow::Result;
use serde::Serialize;
use vx_runtime::ProviderRegistry;

/// Structured output for the search command
#[derive(Serialize)]
pub struct SearchOutput {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total: usize,
}

/// A single search result entry
#[derive(Serialize)]
pub struct SearchResult {
    pub name: String,
    pub description: String,
}

impl CommandOutput for SearchOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer)?;
        if self.results.is_empty() {
            writeln!(writer, "  No runtimes found matching '{}'", self.query)?;
        } else {
            for result in &self.results {
                writeln!(writer, "  {} - {}", result.name, result.description)?;
            }
        }
        Ok(())
    }
}

pub async fn handle(
    registry: &ProviderRegistry,
    query: Option<String>,
    _category: Option<String>,
    _installed_only: bool,
    _available_only: bool,
    format: OutputFormat,
    _verbose: bool,
) -> Result<()> {
    let query = query.unwrap_or_default();
    let query_lower = query.to_lowercase();

    // Collect results
    let mut results = Vec::new();
    for name in registry.runtime_names() {
        if (query.is_empty() || name.to_lowercase().contains(&query_lower))
            && let Some(runtime) = registry.get_runtime(&name)
        {
            let description = runtime.description().to_string();
            if query.is_empty()
                || description.to_lowercase().contains(&query_lower)
                || name.to_lowercase().contains(&query_lower)
            {
                results.push(SearchResult { name, description });
            }
        }
    }

    let total = results.len();
    let output = SearchOutput {
        query: query.clone(),
        results,
        total,
    };

    // Use unified output renderer
    let renderer = OutputRenderer::new(format);
    if renderer.is_json() {
        renderer.render(&output)?;
    } else {
        // Text mode: use existing UI for header, then render results
        UI::header(&format!("Searching for '{}'...", query));
        renderer.render(&output)?;
    }

    Ok(())
}
