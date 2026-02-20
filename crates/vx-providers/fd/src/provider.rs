//! fd provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// fd provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct FdProvider;

impl Provider for FdProvider {
    fn name(&self) -> &str {
        "fd"
    }

    fn description(&self) -> &str {
        "A simple, fast and user-friendly alternative to find"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("fd", "fd", ProviderSource::BuiltIn)
                .with_description("A simple, fast and user-friendly alternative to find"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FdProvider)
}
