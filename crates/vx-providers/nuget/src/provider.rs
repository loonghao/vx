//! nuget provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// nuget provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct NugetProvider;

impl Provider for NugetProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("nuget")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("NuGet - .NET package manager")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("nuget", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NugetProvider)
}
