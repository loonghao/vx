//! ninja provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// ninja provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct NinjaProvider;

impl Provider for NinjaProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("ninja")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Ninja - a small build system with a focus on speed")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("ninja", "ninja", ProviderSource::BuiltIn)
                .with_description("Ninja - a small build system with a focus on speed")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "ninja",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NinjaProvider)
}
