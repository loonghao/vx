//! ninja provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// ninja provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct NinjaProvider;

impl Provider for NinjaProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("ninja")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Ninja - A small build system")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("ninja", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NinjaProvider)
}
