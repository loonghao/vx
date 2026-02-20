//! gh provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// gh provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct GitHubCliProvider;

impl Provider for GitHubCliProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("gh")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("GitHub CLI - command line tool for GitHub")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("gh", "gh", ProviderSource::BuiltIn).with_fetch_versions(
                vx_starlark::make_fetch_versions_fn("gh", crate::PROVIDER_STAR),
            ),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GitHubCliProvider)
}
