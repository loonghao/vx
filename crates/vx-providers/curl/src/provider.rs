//! curl provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// curl provider (Starlark-driven, system detection only)
#[derive(Debug, Default)]
pub struct CurlProvider;

impl Provider for CurlProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("curl")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Command-line tool for transferring data with URLs")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("curl", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(CurlProvider)
}
