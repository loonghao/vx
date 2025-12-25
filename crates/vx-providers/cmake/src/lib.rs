//! CMake provider for vx
//!
//! This crate provides the CMake build system provider for vx.
//! CMake is a cross-platform build system generator.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_cmake::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "cmake");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::CMakeUrlBuilder;
pub use provider::CMakeProvider;
pub use runtime::CMakeRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new CMake provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(CMakeProvider::new())
}
