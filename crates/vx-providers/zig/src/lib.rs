//! Zig programming language provider for vx
//!
//! This crate provides Zig runtime support using the vx-runtime traits.
//! Zig is a general-purpose programming language and toolchain.

pub mod config;
mod provider;
mod runtime;

pub use config::ZigConfig;
pub use provider::ZigProvider;
pub use runtime::ZigRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Zig provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ZigProvider::new())
}
