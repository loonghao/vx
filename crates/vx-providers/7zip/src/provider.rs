//! 7zip provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// 7zip provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct SevenZipProvider;

impl Provider for SevenZipProvider {
    fn name(&self) -> &str {
        "7zip"
    }

    fn description(&self) -> &str {
        "7-Zip file archiver with high compression ratio"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "7zip",
            "7zip",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(SevenZipProvider)
}
