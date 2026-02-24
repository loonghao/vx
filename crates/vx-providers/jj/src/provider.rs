//! jj provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// jj provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JjProvider;

impl Provider for JjProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("jj")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Jujutsu - A Git-compatible VCS")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("jj", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JjProvider)
}
