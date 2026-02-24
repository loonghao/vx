//! gh provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// gh provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct GhProvider;

impl Provider for GhProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("gh")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("GitHub CLI")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("gh", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GhProvider)
}
