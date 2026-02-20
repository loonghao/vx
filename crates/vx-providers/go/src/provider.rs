//! go provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// go provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct GoProvider;

impl Provider for GoProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("go")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Go programming language toolchain")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("go", "go", ProviderSource::BuiltIn).with_fetch_versions(
                vx_starlark::make_fetch_versions_fn("go", crate::PROVIDER_STAR),
            ),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(GoProvider)
}
