//! ollama provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// ollama provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct OllamaProvider;

impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn description(&self) -> &str {
        "Get up and running with large language models locally"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("ollama", "ollama", ProviderSource::BuiltIn)
                .with_description("Get up and running with large language models locally"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(OllamaProvider)
}
