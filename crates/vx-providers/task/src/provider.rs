//! task provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// task provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct TaskProvider;

impl Provider for TaskProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("task")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("A task runner / simpler Make alternative")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("task", "task", ProviderSource::BuiltIn)
                .with_description("A task runner / simpler Make alternative")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "task",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(TaskProvider)
}
