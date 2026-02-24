//! bat provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// bat provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct BatProvider;

impl Provider for BatProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("bat")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("bat - A cat clone with syntax highlighting")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("bat", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(BatProvider)
}
