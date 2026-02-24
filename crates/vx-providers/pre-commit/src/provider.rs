//! pre-commit provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// pre-commit provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PreCommitProvider;

impl Provider for PreCommitProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("pre-commit")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or(
            "A framework for managing and maintaining multi-language pre-commit hooks",
        )
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("pre-commit", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PreCommitProvider)
}
