//! gh provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// gh provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct GitHubCliProvider;

impl Provider for GitHubCliProvider {
    fn name(&self) -> &str {
        "gh"
    }

    fn description(&self) -> &str {
        "GitHub CLI - command line tool for GitHub"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "gh",
            "gh",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GitHubCliProvider)
}
