//! bun provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// bun provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct BunProvider;

impl Provider for BunProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("bun")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or(
            "Incredibly fast JavaScript runtime, bundler, test runner, and package manager",
        )
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(
                ManifestDrivenRuntime::new("bun", "bun", ProviderSource::BuiltIn)
                    .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                        "bun",
                        crate::PROVIDER_STAR,
                    )),
            ),
            Arc::new(ManifestDrivenRuntime::new(
                "bunx",
                "bunx",
                ProviderSource::BuiltIn,
            )),
        ]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(BunProvider)
}
