//! terraform provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// terraform provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct TerraformProvider;

impl Provider for TerraformProvider {
    fn name(&self) -> &str {
        "terraform"
    }

    fn description(&self) -> &str {
        "HashiCorp Terraform - Infrastructure as Code"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "terraform",
            "terraform",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(TerraformProvider)
}
