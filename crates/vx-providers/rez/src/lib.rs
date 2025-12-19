//! Rez provider for vx
//!
//! This crate provides the Rez package manager provider for vx.
//! Rez is a cross-platform package manager with a difference - it
//! provides a deterministic environment for software development.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_rez::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "rez");
//! ```

mod provider;
mod runtime;

pub use provider::RezProvider;
pub use runtime::RezRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Rez provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RezProvider::new())
}
