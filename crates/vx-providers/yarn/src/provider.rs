//! yarn provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// yarn provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct YarnProvider;

impl Provider for YarnProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("yarn")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Fast, reliable, and secure dependency management")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("yarn", crate::PROVIDER_STAR, Some("yarn"))
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(YarnProvider)
}
