//! rust provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// rust provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct RustProvider;

impl Provider for RustProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("rust")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Rust toolchain manager")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("rust", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RustProvider)
}
