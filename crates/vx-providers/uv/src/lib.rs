//! UV provider for vx
//!
//! This provider includes:
//! - UV Python package installer (`uv`)
//! - UVX Python application runner (`uvx`)

mod config;
mod provider;
mod runtime;

pub use config::UvUrlBuilder;
pub use provider::UvProvider;
pub use runtime::{UvRuntime, UvxRuntime};

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new UV provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(UvProvider::new())
}
