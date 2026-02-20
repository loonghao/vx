//! ninja provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// ninja provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct NinjaProvider;

impl Provider for NinjaProvider {
    fn name(&self) -> &str {
        "ninja"
    }

    fn description(&self) -> &str {
        "Ninja - a small build system with a focus on speed"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("ninja", "ninja", ProviderSource::BuiltIn)
                .with_description("Ninja - a small build system with a focus on speed"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NinjaProvider)
}
