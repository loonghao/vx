//! 7zip provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// 7zip provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct SevenZipProvider;

impl Provider for SevenZipProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("7zip")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("7-Zip file archiver")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("7zip", "7zip", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "7zip",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(SevenZipProvider)
}
