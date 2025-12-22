//! Helm (Kubernetes package manager) provider for vx
//!
//! This crate provides Helm runtime support using the vx-runtime traits.
//! Helm is the package manager for Kubernetes.

pub mod config;
mod provider;
mod runtime;

pub use config::HelmConfig;
pub use provider::HelmProvider;
pub use runtime::HelmRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Helm provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(HelmProvider::new())
}
