//! Vite provider for vx
//!
//! This crate provides the Vite frontend build tool provider for vx.
//! Vite is a next-generation frontend build tool that significantly improves
//! the frontend development experience.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_vite::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "vite");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::ViteUrlBuilder;
pub use provider::ViteProvider;
pub use runtime::ViteRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Vite provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ViteProvider::new())
}
