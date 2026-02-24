//! task provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// task provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct TaskProvider;

impl Provider for TaskProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("task")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Task - A task runner / build tool")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("task", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(TaskProvider)
}
