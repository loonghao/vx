//! spack provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// spack provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct SpackProvider;

impl Provider for SpackProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("spack")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Spack - A flexible package manager")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("spack", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(SpackProvider)
}
