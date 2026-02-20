//! java provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// java provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JavaProvider;

impl Provider for JavaProvider {
    fn name(&self) -> &str {
        "java"
    }

    fn description(&self) -> &str {
        "Java Development Kit"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "java",
            "java",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JavaProvider)
}
