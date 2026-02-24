//! ollama provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// ollama provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct OllamaProvider;

impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("ollama")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Ollama - Run large language models locally")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("ollama", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(OllamaProvider)
}
