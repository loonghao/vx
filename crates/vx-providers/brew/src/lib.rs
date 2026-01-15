//! Homebrew package manager provider for vx
//!
//! This crate provides Homebrew package manager support using the vx-runtime traits.
//! Homebrew is the missing package manager for macOS (and Linux).

mod config;
mod provider;
mod runtime;

pub use config::BrewConfig;
pub use provider::BrewProvider;
pub use runtime::BrewRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Homebrew provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(BrewProvider::new())
}
