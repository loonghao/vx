//! dotnet provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// dotnet provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct DotnetProvider;

impl Provider for DotnetProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("dotnet")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or(".NET SDK and runtime")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("dotnet", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DotnetProvider)
}
