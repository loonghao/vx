//! yq provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// yq provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct YqProvider;

impl Provider for YqProvider {
    fn name(&self) -> &str {
        "yq"
    }

    fn description(&self) -> &str {
        "A portable command-line YAML, JSON, XML, CSV, TOML and properties processor"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("yq", "yq", ProviderSource::BuiltIn).with_description(
                "A portable command-line YAML, JSON, XML, CSV, TOML and properties processor",
            ),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(YqProvider)
}
