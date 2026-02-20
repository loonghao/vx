//! azcli provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// azcli provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct AzCliProvider;

impl Provider for AzCliProvider {
    fn name(&self) -> &str {
        "azcli"
    }

    fn description(&self) -> &str {
        "Azure CLI - Command-line interface for Microsoft Azure"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "az",
            "az",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(AzCliProvider)
}
