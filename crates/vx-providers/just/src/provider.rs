//! just provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// just provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JustProvider;

impl Provider for JustProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("just")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("just")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("just", "just", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "just",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JustProvider)
}
