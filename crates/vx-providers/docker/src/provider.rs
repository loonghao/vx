//! docker provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// docker provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct DockerProvider;

impl Provider for DockerProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("docker")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Docker - Container platform")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("docker", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DockerProvider)
}
