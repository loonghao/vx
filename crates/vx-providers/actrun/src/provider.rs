//! actrun provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// actrun provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ActrunProvider;

impl Provider for ActrunProvider {
    fn name(&self) -> &str {
        "actrun"
    }

    fn description(&self) -> &str {
        "Run GitHub Actions locally"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("actrun", "actrun", ProviderSource::BuiltIn)
                .with_description("Run GitHub Actions locally"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ActrunProvider)
}
