//! prek provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// prek provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PrekProvider;

impl Provider for PrekProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("prek")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Pre-commit hook runner")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("prek", crate::PROVIDER_STAR, Some("prek"))
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PrekProvider)
}
