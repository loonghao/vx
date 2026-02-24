//! pwsh provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// pwsh provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PwshProvider;

impl Provider for PwshProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("pwsh")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("PowerShell - Cross-platform task automation")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("pwsh", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PwshProvider)
}
