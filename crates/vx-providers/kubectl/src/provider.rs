//! kubectl provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// kubectl provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct KubectlProvider;

impl Provider for KubectlProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("kubectl")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("kubectl - Kubernetes command-line tool")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("kubectl", "kubectl", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "kubectl",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(KubectlProvider)
}
