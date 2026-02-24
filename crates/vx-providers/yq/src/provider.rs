//! yq provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// yq provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct YqProvider;

impl Provider for YqProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("yq")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("yq - A portable command-line YAML processor")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("yq", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(YqProvider)
}
