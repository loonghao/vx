//! fzf provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// fzf provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct FzfProvider;

impl Provider for FzfProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("fzf")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("fzf - A command-line fuzzy finder")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("fzf", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FzfProvider)
}
