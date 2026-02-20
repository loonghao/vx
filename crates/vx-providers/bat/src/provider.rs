//! bat provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// bat provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct BatProvider;

impl Provider for BatProvider {
    fn name(&self) -> &str {
        "bat"
    }

    fn description(&self) -> &str {
        "A cat clone with syntax highlighting and Git integration"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("bat", "bat", ProviderSource::BuiltIn)
                .with_description("A cat clone with syntax highlighting and Git integration"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(BatProvider)
}
