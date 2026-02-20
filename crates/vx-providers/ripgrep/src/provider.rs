//! ripgrep provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// ripgrep provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct RipgrepProvider;

impl Provider for RipgrepProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("ripgrep")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("ripgrep (rg) - recursively searches directories for a regex pattern")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("ripgrep", "ripgrep", ProviderSource::BuiltIn)
                .with_description(
                    "ripgrep (rg) - recursively searches directories for a regex pattern",
                )
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "ripgrep",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RipgrepProvider)
}
