//! cmake provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// cmake provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct CMakeProvider;

impl Provider for CMakeProvider {
    fn name(&self) -> &str {
        "cmake"
    }

    fn description(&self) -> &str {
        "Cross-platform build system generator"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "cmake",
            "cmake",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(CMakeProvider)
}
