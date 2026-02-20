//! helm provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// helm provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct HelmProvider;

impl Provider for HelmProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("helm")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("The Kubernetes Package Manager")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("helm", "helm", ProviderSource::BuiltIn)
                .with_description("The Kubernetes Package Manager")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "helm",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(HelmProvider)
}
