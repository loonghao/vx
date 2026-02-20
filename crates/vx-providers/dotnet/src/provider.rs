//! dotnet provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// dotnet provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct DotnetProvider;

impl Provider for DotnetProvider {
    fn name(&self) -> &str {
        "dotnet"
    }

    fn description(&self) -> &str {
        ".NET SDK and runtime"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "dotnet",
            "dotnet",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DotnetProvider)
}
