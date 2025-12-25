//! Ninja provider for vx
//!
//! This crate provides the Ninja build system provider for vx.
//! Ninja is a small build system with a focus on speed.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_ninja::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "ninja");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::NinjaUrlBuilder;
pub use provider::NinjaProvider;
pub use runtime::NinjaRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Ninja provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NinjaProvider::new())
}
