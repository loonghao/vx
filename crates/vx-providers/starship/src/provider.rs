//! starship provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// starship provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct StarshipProvider;

impl Provider for StarshipProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("starship")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Starship - The minimal, blazing-fast shell prompt")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("starship", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(StarshipProvider)
}
