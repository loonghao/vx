//! bat provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// bat provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct BatProvider;

impl Provider for BatProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("bat")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("A cat clone with syntax highlighting and Git integration")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("bat", "bat", ProviderSource::BuiltIn)
                .with_description("A cat clone with syntax highlighting and Git integration")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "bat",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(BatProvider)
}
