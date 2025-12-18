//! Just provider for vx
//!
//! This crate provides the Just command runner provider for vx.
//! Just is a handy way to save and run project-specific commands.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_just::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "just");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::JustUrlBuilder;
pub use provider::JustProvider;
pub use runtime::JustRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Just provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JustProvider::new())
}
