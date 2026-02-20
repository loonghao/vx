//! python provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// python provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct PythonProvider;

impl Provider for PythonProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("python")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Python programming language")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("python", "python", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "python",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PythonProvider)
}
