//! kubectl (Kubernetes CLI) provider for vx
//!
//! This crate provides kubectl runtime support using the vx-runtime traits.
//! kubectl is the Kubernetes command-line tool.

pub mod config;
mod provider;
mod runtime;

pub use config::KubectlConfig;
pub use provider::KubectlProvider;
pub use runtime::KubectlRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new kubectl provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(KubectlProvider::new())
}
