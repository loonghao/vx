//! prek provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// prek provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PrekProvider;

impl Provider for PrekProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("prek")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Pre-commit hook runner")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("prek", "prek", ProviderSource::BuiltIn)
                .with_description("Pre-commit hook runner")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "prek",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PrekProvider)
}
