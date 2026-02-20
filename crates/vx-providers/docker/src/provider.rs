//! docker provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// docker provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct DockerProvider;

impl Provider for DockerProvider {
    fn name(&self) -> &str {
        "docker"
    }

    fn description(&self) -> &str {
        "Docker container platform"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "docker",
            "docker",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DockerProvider)
}
