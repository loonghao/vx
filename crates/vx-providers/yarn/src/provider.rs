//! yarn provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// yarn provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct YarnProvider;

impl Provider for YarnProvider {
    fn name(&self) -> &str {
        "yarn"
    }

    fn description(&self) -> &str {
        "Fast, reliable, and secure dependency management"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "yarn",
            "yarn",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(YarnProvider)
}
