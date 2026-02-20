//! ollama provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// ollama provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct OllamaProvider;

impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("ollama")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("Get up and running with large language models locally")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("ollama", "ollama", ProviderSource::BuiltIn)
                .with_description("Get up and running with large language models locally")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "ollama",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(OllamaProvider)
}
