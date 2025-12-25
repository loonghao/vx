//! Ollama provider for vx
//!
//! This crate provides the Ollama provider for vx.
//! Ollama enables running large language models locally, including Llama, Mistral,
//! Gemma, and many other open-source models.
//!
//! # Features
//!
//! - Install Ollama on Windows, macOS, and Linux
//! - Version management
//! - Support for local LLM development and testing
//! - GPU acceleration support (CUDA, ROCm)
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_ollama::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "ollama");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::OllamaUrlBuilder;
pub use provider::OllamaProvider;
pub use runtime::OllamaRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Ollama provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(OllamaProvider::new())
}
