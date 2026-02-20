//! pnpm provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// pnpm provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PnpmProvider;

impl Provider for PnpmProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("pnpm")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Fast, disk space efficient package manager")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(
                ManifestDrivenRuntime::new("pnpm", "pnpm", ProviderSource::BuiltIn)
                    .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                        "pnpm",
                        crate::PROVIDER_STAR,
                    )),
            ),
            Arc::new(ManifestDrivenRuntime::new(
                "pnpx",
                "pnpx",
                ProviderSource::BuiltIn,
            )),
        ]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PnpmProvider)
}
