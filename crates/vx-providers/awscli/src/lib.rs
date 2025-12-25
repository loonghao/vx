//! AWS CLI provider for vx
//!
//! This crate provides AWS CLI v2 runtime support using the vx-runtime traits.
//! AWS CLI is the unified command line interface to Amazon Web Services.

pub mod config;
mod provider;
mod runtime;

pub use config::AwsCliConfig;
pub use provider::AwsCliProvider;
pub use runtime::AwsCliRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new AWS CLI provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(AwsCliProvider::new())
}
