//! bun provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// bun provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct BunProvider;

impl Provider for BunProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("bun")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or(
            "Incredibly fast JavaScript runtime, bundler, test runner, and package manager",
        )
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("bun", crate::PROVIDER_STAR, Some("bun"))
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(BunProvider)
}
