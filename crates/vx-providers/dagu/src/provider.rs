//! dagu provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// dagu provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct DaguProvider;

impl Provider for DaguProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("dagu")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Dagu - A No-code workflow runner")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("dagu", "dagu", ProviderSource::BuiltIn)
                .with_description("Dagu - A No-code workflow runner")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "dagu",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DaguProvider)
}
