//! Google Cloud CLI provider for vx
//!
//! This crate provides Google Cloud SDK/gcloud CLI runtime support using the vx-runtime traits.
//! gcloud is the command-line interface for Google Cloud Platform.

pub mod config;
mod provider;
mod runtime;

pub use config::{GcloudConfig, GcloudUrlBuilder};
pub use provider::GcloudProvider;
pub use runtime::GcloudRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new gcloud provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GcloudProvider::new())
}
