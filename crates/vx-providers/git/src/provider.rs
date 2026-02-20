//! git provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// git provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct GitProvider;

impl Provider for GitProvider {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "Git - distributed version control system"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "git",
            "git",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GitProvider)
}
