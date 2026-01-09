//! Bun runtime provider for vx
//!
//! This crate provides Bun runtime support using the new vx-runtime traits.
//! Bun is an incredibly fast JavaScript runtime, bundler, test runner, and package manager.

mod config;
mod provider;
mod runtime;

pub use config::BunConfig;
pub use provider::BunProvider;
pub use runtime::{BunRuntime, BunxRuntime};

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Bun provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(BunProvider::new())
}
