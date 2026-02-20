//! prek provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// prek provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PrekProvider;

impl Provider for PrekProvider {
    fn name(&self) -> &str {
        "prek"
    }

    fn description(&self) -> &str {
        "Pre-commit hook runner"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("prek", "prek", ProviderSource::BuiltIn)
                .with_description("Pre-commit hook runner"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PrekProvider)
}
