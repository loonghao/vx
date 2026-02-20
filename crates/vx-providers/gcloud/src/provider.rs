//! gcloud provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// gcloud provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct GcloudProvider;

impl Provider for GcloudProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("gcloud")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Google Cloud CLI")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("gcloud", "gcloud", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "gcloud",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GcloudProvider)
}
