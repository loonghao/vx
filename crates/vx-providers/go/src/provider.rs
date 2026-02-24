//! go provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// go provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct GoProvider;

impl Provider for GoProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("go")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Go programming language toolchain")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("go", crate::PROVIDER_STAR, Some("go"))
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GoProvider)
}
