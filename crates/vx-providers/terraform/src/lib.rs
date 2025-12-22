//! Terraform provider for vx
//!
//! This crate provides Terraform runtime support using the vx-runtime traits.
//! Terraform is an infrastructure as code tool by HashiCorp.

pub mod config;
mod provider;
mod runtime;

pub use config::TerraformConfig;
pub use provider::TerraformProvider;
pub use runtime::TerraformRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Terraform provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(TerraformProvider::new())
}
