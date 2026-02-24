//! helm provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// helm provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct HelmProvider;

impl Provider for HelmProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("helm")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Helm - The Kubernetes Package Manager")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("helm", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(HelmProvider)
}
