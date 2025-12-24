//! Git provider for vx.
//!
//! This crate provides Git version control system support for vx,
//! allowing users to install and manage Git versions.

mod config;
mod provider;
mod runtime;

pub use config::GitUrlBuilder;
pub use provider::GitProvider;
pub use runtime::GitRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Git provider instance.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GitProvider::new())
}
