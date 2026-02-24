//! fd provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// fd provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct FdProvider;

impl Provider for FdProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("fd")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("fd - A simple, fast and user-friendly alternative to find")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("fd", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FdProvider)
}
