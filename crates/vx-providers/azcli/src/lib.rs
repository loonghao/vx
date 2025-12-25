//! Azure CLI provider for vx
//!
//! This crate provides Azure CLI runtime support using the vx-runtime traits.
//! Azure CLI is the command-line interface for Microsoft Azure.

pub mod config;
mod provider;
mod runtime;

pub use config::AzCliConfig;
pub use provider::AzCliProvider;
pub use runtime::AzCliRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Azure CLI provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(AzCliProvider::new())
}
