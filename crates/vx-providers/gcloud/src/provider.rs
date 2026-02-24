//! gcloud provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

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
        vx_starlark::build_runtimes("gcloud", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GcloudProvider)
}
