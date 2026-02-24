//! azcli provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// azcli provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct AzCliProvider;

impl Provider for AzCliProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("azcli")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Azure CLI - Microsoft Azure command line interface")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("azcli", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(AzCliProvider)
}
