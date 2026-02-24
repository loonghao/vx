//! terraform provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// terraform provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct TerraformProvider;

impl Provider for TerraformProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("terraform")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Terraform - Infrastructure as Code")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("terraform", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(TerraformProvider)
}
