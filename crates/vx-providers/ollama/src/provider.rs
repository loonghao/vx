//! Ollama provider implementation
//!
//! Provides the Ollama runtime for local LLM support.

use crate::runtime::OllamaRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Ollama provider
#[derive(Debug, Default)]
pub struct OllamaProvider;

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn description(&self) -> &str {
        "Ollama - Run large language models locally"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(OllamaRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "ollama"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "ollama" {
            Some(Arc::new(OllamaRuntime::new()))
        } else {
            None
        }
    }
}
