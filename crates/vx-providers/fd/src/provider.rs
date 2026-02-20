//! fd provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// fd provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct FdProvider;

impl Provider for FdProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("fd")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("A simple, fast and user-friendly alternative to find")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("fd", "fd", ProviderSource::BuiltIn)
                .with_description("A simple, fast and user-friendly alternative to find")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "fd",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FdProvider)
}
