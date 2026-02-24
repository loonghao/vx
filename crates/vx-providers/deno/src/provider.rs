//! deno provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// deno provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct DenoProvider;

impl Provider for DenoProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("deno")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("Deno - A modern runtime for JavaScript and TypeScript")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("deno", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DenoProvider)
}
