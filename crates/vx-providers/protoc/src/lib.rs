//! Protoc provider for vx
//!
//! This crate provides the Protocol Buffers compiler (protoc) provider for vx.
//! protoc is the compiler for Protocol Buffers, Google's data interchange format.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_protoc::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "protoc");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::ProtocUrlBuilder;
pub use provider::ProtocProvider;
pub use runtime::ProtocRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new protoc provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ProtocProvider::new())
}
