//! docker provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// docker provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct DockerProvider;

impl Provider for DockerProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("docker")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Docker container platform")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("docker", "docker", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "docker",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DockerProvider)
}
