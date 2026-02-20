//! azcli provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// azcli provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct AzCliProvider;

impl Provider for AzCliProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("azcli")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("Azure CLI - Command-line interface for Microsoft Azure")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("az", "az", ProviderSource::BuiltIn).with_fetch_versions(
                vx_starlark::make_fetch_versions_fn("azcli", crate::PROVIDER_STAR),
            ),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(AzCliProvider)
}
