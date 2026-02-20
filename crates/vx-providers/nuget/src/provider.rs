//! nuget provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// nuget provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct NuGetProvider;

impl Provider for NuGetProvider {
    fn name(&self) -> &str {
        "nuget"
    }

    fn description(&self) -> &str {
        "NuGet package manager for .NET"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "nuget",
            "nuget",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NuGetProvider)
}
