//! Chocolatey package manager provider for vx
//!
//! This crate provides Chocolatey package manager support using the vx-runtime traits.
//! Chocolatey is the package manager for Windows, providing easy installation of
//! software, tools, and libraries.

mod config;
mod provider;
mod runtime;

pub use config::ChocoConfig;
pub use provider::ChocoProvider;
pub use runtime::ChocoRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Chocolatey provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ChocoProvider::new())
}
