//! nuget provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// nuget provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct NuGetProvider;

impl Provider for NuGetProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("nuget")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("NuGet package manager for .NET")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("nuget", "nuget", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "nuget",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NuGetProvider)
}
