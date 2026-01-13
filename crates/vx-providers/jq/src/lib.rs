//! jq provider for vx
//!
//! This crate provides the jq JSON processor provider for vx.
//! jq is a lightweight and flexible command-line JSON processor.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_jq::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "jq");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::JqUrlBuilder;
pub use provider::JqProvider;
pub use runtime::JqRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new jq provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JqProvider::new())
}
