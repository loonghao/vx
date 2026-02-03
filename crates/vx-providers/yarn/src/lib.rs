//! Yarn package manager provider for vx
//!
//! This crate provides Yarn runtime support using the new vx-runtime traits.

mod config;
mod provider;
mod runtime;

pub use config::{YarnConfig, YarnUrlBuilder};
pub use provider::YarnProvider;
pub use runtime::YarnRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Yarn provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(YarnProvider::new())
}
