//! uv provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// uv provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct UvProvider;

impl Provider for UvProvider {
    fn name(&self) -> &str {
        "uv"
    }

    fn description(&self) -> &str {
        "An extremely fast Python package installer and resolver"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(ManifestDrivenRuntime::new(
                "uv",
                "uv",
                ProviderSource::BuiltIn,
            )),
            Arc::new(ManifestDrivenRuntime::new(
                "uvx",
                "uvx",
                ProviderSource::BuiltIn,
            )),
        ]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(UvProvider)
}
