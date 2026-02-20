//! just provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// just provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JustProvider;

impl Provider for JustProvider {
    fn name(&self) -> &str {
        "just"
    }

    fn description(&self) -> &str {
        "A handy way to save and run project-specific commands"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("just", "just", ProviderSource::BuiltIn)
                .with_description("A handy way to save and run project-specific commands"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JustProvider)
}
