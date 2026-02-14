use std::sync::Arc;
use vx_runtime::Provider;

use crate::runtime::PrekRuntime;

/// prek Provider implementation
#[derive(Debug, Clone, Default)]
pub struct PrekProvider;

impl PrekProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for PrekProvider {
    fn name(&self) -> &str {
        "prek"
    }

    fn description(&self) -> &str {
        "prek - better pre-commit, re-engineered in Rust"
    }

    fn runtimes(&self) -> Vec<Arc<dyn vx_runtime::Runtime>> {
        vec![Arc::new(PrekRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "prek"
    }
}
