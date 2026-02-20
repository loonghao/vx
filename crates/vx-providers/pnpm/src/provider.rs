//! pnpm provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// pnpm provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PnpmProvider;

impl Provider for PnpmProvider {
    fn name(&self) -> &str {
        "pnpm"
    }

    fn description(&self) -> &str {
        "Fast, disk space efficient package manager"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(ManifestDrivenRuntime::new(
                "pnpm",
                "pnpm",
                ProviderSource::BuiltIn,
            )),
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
