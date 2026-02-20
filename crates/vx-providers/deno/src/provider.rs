//! deno provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// deno provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct DenoProvider;

impl Provider for DenoProvider {
    fn name(&self) -> &str {
        "deno"
    }

    fn description(&self) -> &str {
        "Deno - A modern runtime for JavaScript and TypeScript"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("deno", "deno", ProviderSource::BuiltIn)
                .with_description("Deno - A modern runtime for JavaScript and TypeScript"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DenoProvider)
}
