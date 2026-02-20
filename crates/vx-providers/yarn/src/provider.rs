//! yarn provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// yarn provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct YarnProvider;

impl Provider for YarnProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("yarn")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Fast, reliable, and secure dependency management")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("yarn", "yarn", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "yarn",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(YarnProvider)
}
