//! spack provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// spack provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct SpackProvider;

impl Provider for SpackProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("spack")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("A flexible package manager for HPC")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("spack", "spack", ProviderSource::BuiltIn)
                .with_description("A flexible package manager for HPC")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "spack",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(SpackProvider)
}
