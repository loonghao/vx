//! deno provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// deno provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct DenoProvider;

impl Provider for DenoProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("deno")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("Deno - A modern runtime for JavaScript and TypeScript")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("deno", "deno", ProviderSource::BuiltIn)
                .with_description("Deno - A modern runtime for JavaScript and TypeScript")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "deno",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DenoProvider)
}
