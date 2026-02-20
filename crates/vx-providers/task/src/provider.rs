//! task provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// task provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct TaskProvider;

impl Provider for TaskProvider {
    fn name(&self) -> &str {
        "task"
    }

    fn description(&self) -> &str {
        "A task runner / simpler Make alternative"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("task", "task", ProviderSource::BuiltIn)
                .with_description("A task runner / simpler Make alternative"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(TaskProvider)
}
