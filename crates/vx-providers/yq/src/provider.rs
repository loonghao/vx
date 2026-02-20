//! yq provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// yq provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct YqProvider;

impl Provider for YqProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("yq")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or(
            "A portable command-line YAML, JSON, XML, CSV, TOML and properties processor",
        )
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("yq", "yq", ProviderSource::BuiltIn)
                .with_description(
                    "A portable command-line YAML, JSON, XML, CSV, TOML and properties processor",
                )
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "yq",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(YqProvider)
}
