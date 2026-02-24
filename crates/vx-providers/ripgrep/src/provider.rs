//! ripgrep provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// ripgrep provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct RipgrepProvider;

impl Provider for RipgrepProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("ripgrep")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("ripgrep - Recursively searches directories for a regex pattern")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("ripgrep", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RipgrepProvider)
}
