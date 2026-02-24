//! cmake provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// cmake provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct CmakeProvider;

impl Provider for CmakeProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("cmake")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("CMake - Cross-platform build system")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("cmake", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(CmakeProvider)
}
