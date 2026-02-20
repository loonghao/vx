//! fzf provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// fzf provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct FzfProvider;

impl Provider for FzfProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("fzf")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("A command-line fuzzy finder")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("fzf", "fzf", ProviderSource::BuiltIn)
                .with_description("A command-line fuzzy finder")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "fzf",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FzfProvider)
}
