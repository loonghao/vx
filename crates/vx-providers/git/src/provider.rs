//! git provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// git provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct GitProvider;

impl Provider for GitProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("git")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Git - distributed version control system")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("git", crate::PROVIDER_STAR, Some("git"))
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GitProvider)
}
