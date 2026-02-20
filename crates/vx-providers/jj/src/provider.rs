//! jj provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// jj provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JujutsuProvider;

impl Provider for JujutsuProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("jj")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Jujutsu - a Git-compatible DVCS")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("jj", "jj", ProviderSource::BuiltIn).with_fetch_versions(
                vx_starlark::make_fetch_versions_fn("jj", crate::PROVIDER_STAR),
            ),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JujutsuProvider)
}
