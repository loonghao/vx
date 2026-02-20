//! jj provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// jj provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JujutsuProvider;

impl Provider for JujutsuProvider {
    fn name(&self) -> &str {
        "jj"
    }

    fn description(&self) -> &str {
        "Jujutsu - a Git-compatible DVCS"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "jj",
            "jj",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JujutsuProvider)
}
