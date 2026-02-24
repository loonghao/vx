//! release-please provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// release-please provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ReleasePleaseProvider;

impl Provider for ReleasePleaseProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("release-please")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Automated release PRs based on Conventional Commits")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("release-please", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ReleasePleaseProvider)
}
