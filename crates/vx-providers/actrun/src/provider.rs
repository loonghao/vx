//! actrun provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// actrun provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ActrunProvider;

impl Provider for ActrunProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("actrun")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Run GitHub Actions locally")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("actrun", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ActrunProvider)
}
