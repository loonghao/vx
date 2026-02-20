//! kubectl provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// kubectl provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct KubectlProvider;

impl Provider for KubectlProvider {
    fn name(&self) -> &str {
        "kubectl"
    }

    fn description(&self) -> &str {
        "kubectl - Kubernetes command-line tool"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "kubectl",
            "kubectl",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(KubectlProvider)
}
