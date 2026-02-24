//! hadolint provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// hadolint provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct HadolintProvider;

impl Provider for HadolintProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("hadolint")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Dockerfile linter")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("hadolint", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(HadolintProvider)
}
