//! starship provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// starship provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct StarshipProvider;

impl Provider for StarshipProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("starship")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("The minimal, blazing-fast, and infinitely customizable prompt")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("starship", "starship", ProviderSource::BuiltIn)
                .with_description("The minimal, blazing-fast, and infinitely customizable prompt")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "starship",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(StarshipProvider)
}
