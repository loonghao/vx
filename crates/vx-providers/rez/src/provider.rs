//! rez provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// rez provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct RezProvider;

impl Provider for RezProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("rez")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("Cross-platform package manager for deterministic environments")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("rez", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RezProvider)
}
