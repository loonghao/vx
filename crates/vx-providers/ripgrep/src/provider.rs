//! ripgrep provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// ripgrep provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct RipgrepProvider;

impl Provider for RipgrepProvider {
    fn name(&self) -> &str {
        "ripgrep"
    }

    fn description(&self) -> &str {
        "ripgrep (rg) - recursively searches directories for a regex pattern"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("ripgrep", "ripgrep", ProviderSource::BuiltIn)
                .with_description(
                    "ripgrep (rg) - recursively searches directories for a regex pattern",
                ),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RipgrepProvider)
}
