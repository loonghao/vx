//! gcloud provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// gcloud provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct GcloudProvider;

impl Provider for GcloudProvider {
    fn name(&self) -> &str {
        "gcloud"
    }

    fn description(&self) -> &str {
        "Google Cloud CLI"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "gcloud",
            "gcloud",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GcloudProvider)
}
