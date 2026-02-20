//! rcedit provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// rcedit provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct RceditProvider;

impl Provider for RceditProvider {
    fn name(&self) -> &str {
        "rcedit"
    }

    fn description(&self) -> &str {
        "Edit resources of exe files on Windows"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "rcedit",
            "rcedit",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RceditProvider)
}
