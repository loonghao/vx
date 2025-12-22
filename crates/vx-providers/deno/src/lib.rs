//! Deno runtime provider for vx
//!
//! This crate provides Deno runtime support using the vx-runtime traits.
//! Deno is a secure runtime for JavaScript and TypeScript.

pub mod config;
mod provider;
mod runtime;

pub use config::DenoConfig;
pub use provider::DenoProvider;
pub use runtime::DenoRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Deno provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DenoProvider::new())
}
