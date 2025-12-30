//! Rust toolchain provider for vx
//!
//! This crate provides Rust toolchain support using the new vx-runtime traits.

mod config;
mod provider;
mod runtime;

pub use config::{RustConfig, RustUrlBuilder};
pub use provider::RustProvider;
pub use runtime::{CargoRuntime, RustcRuntime, RustupRuntime};

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Rust provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RustProvider::new())
}
