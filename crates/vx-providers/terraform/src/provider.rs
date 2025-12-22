//! Terraform provider implementation

use crate::runtime::TerraformRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Terraform provider
#[derive(Debug)]
pub struct TerraformProvider;

impl TerraformProvider {
    /// Create a new Terraform provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for TerraformProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for TerraformProvider {
    fn name(&self) -> &str {
        "terraform"
    }

    fn description(&self) -> &str {
        "Provides Terraform infrastructure as code tool support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(TerraformRuntime::new())]
    }
}
