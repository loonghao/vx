//! jq provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// jq provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JqProvider;

impl Provider for JqProvider {
    fn name(&self) -> &str {
        "jq"
    }

    fn description(&self) -> &str {
        "A lightweight and flexible command-line JSON processor"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "jq",
            "jq",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JqProvider)
}
