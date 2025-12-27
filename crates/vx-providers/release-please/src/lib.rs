//! release-please provider for vx
//!
//! This crate provides the release-please CLI tool provider for vx.
//! release-please is a Google tool for automating releases based on
//! conventional commits.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_release_please::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "release-please");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::ReleasePleaseUrlBuilder;
pub use provider::ReleasePleaseProvider;
pub use runtime::ReleasePleaseRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new release-please provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ReleasePleaseProvider::new())
}
