//! python provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// python provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PythonProvider;

impl Provider for PythonProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("python")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Python programming language")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("python", crate::PROVIDER_STAR, Some("python"))
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PythonProvider)
}
