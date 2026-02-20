//! git provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

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
        vec![Arc::new(
            ManifestDrivenRuntime::new("git", "git", ProviderSource::BuiltIn).with_fetch_versions(
                vx_starlark::make_fetch_versions_fn("git", crate::PROVIDER_STAR),
            ),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GitProvider)
}
