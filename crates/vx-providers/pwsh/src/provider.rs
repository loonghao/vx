//! pwsh provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// pwsh provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PwshProvider;

impl Provider for PwshProvider {
    fn name(&self) -> &str {
        "pwsh"
    }

    fn description(&self) -> &str {
        "Cross-platform command-line shell and scripting language"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "pwsh",
            "pwsh",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PwshProvider)
}
