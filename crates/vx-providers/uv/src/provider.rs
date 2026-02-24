//! uv provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// uv provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct UvProvider;

impl Provider for UvProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("uv")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("An extremely fast Python package installer and resolver")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("uv", crate::PROVIDER_STAR, Some("uv"))
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(UvProvider)
}
