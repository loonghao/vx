//! spack provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// spack provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct SpackProvider;

impl Provider for SpackProvider {
    fn name(&self) -> &str {
        "spack"
    }

    fn description(&self) -> &str {
        "A flexible package manager for HPC"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("spack", "spack", ProviderSource::BuiltIn)
                .with_description("A flexible package manager for HPC"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(SpackProvider)
}
