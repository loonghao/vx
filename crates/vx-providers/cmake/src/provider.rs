//! cmake provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// cmake provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct CMakeProvider;

impl Provider for CMakeProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("cmake")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Cross-platform build system generator")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("cmake", "cmake", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "cmake",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(CMakeProvider)
}
