use std::sync::Arc;
use vx_runtime::Provider;

use crate::runtime::DaguRuntime;

/// Dagu Provider implementation
#[derive(Debug, Clone, Default)]
pub struct DaguProvider;

impl DaguProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for DaguProvider {
    fn name(&self) -> &str {
        "dagu"
    }

    fn description(&self) -> &str {
        "Dagu - self-contained workflow engine with Web UI"
    }

    fn runtimes(&self) -> Vec<Arc<dyn vx_runtime::Runtime>> {
        vec![Arc::new(DaguRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "dagu"
    }
}
