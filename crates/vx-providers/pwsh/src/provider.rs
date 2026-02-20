//! pwsh provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// pwsh provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PwshProvider;

impl Provider for PwshProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("pwsh")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("Cross-platform command-line shell and scripting language")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("pwsh", "pwsh", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "pwsh",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PwshProvider)
}
