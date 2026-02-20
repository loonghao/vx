//! rcedit provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// rcedit provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct RceditProvider;

impl Provider for RceditProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("rcedit")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Edit resources of exe files on Windows")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("rcedit", "rcedit", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "rcedit",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RceditProvider)
}
