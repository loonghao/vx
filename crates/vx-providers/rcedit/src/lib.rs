//! rcedit provider for vx
//!
//! This crate provides the rcedit (Windows resource editor) provider for vx.
//! rcedit is a command-line tool to edit resources of Windows executables.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_rcedit::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "rcedit");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::RceditUrlBuilder;
pub use provider::RceditProvider;
pub use runtime::RceditRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new rcedit provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RceditProvider::new())
}
