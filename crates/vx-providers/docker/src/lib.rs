//! Docker CLI provider for vx
//!
//! This crate provides Docker CLI runtime support using the vx-runtime traits.
//! Docker is the industry-standard container runtime.

pub mod config;
mod provider;
mod runtime;

pub use config::DockerConfig;
pub use provider::DockerProvider;
pub use runtime::DockerRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Docker provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DockerProvider::new())
}
