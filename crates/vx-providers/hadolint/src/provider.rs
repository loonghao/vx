//! hadolint provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// hadolint provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct HadolintProvider;

impl Provider for HadolintProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("hadolint")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Dockerfile linter")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("hadolint", "hadolint", ProviderSource::BuiltIn)
                .with_description("Dockerfile linter")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "hadolint",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(HadolintProvider)
}
