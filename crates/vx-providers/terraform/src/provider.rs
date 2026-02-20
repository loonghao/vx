//! terraform provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// terraform provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct TerraformProvider;

impl Provider for TerraformProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("terraform")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("HashiCorp Terraform - Infrastructure as Code")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("terraform", "terraform", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "terraform",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(TerraformProvider)
}
