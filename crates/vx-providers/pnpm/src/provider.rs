//! pnpm provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// pnpm provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PnpmProvider;

impl Provider for PnpmProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("pnpm")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("pnpm - Fast, disk space efficient package manager")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("pnpm", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PnpmProvider)
}
