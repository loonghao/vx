//! pipx provider implementation

use crate::runtime::PipxRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// pipx provider
#[derive(Debug, Default)]
pub struct PipxProvider;

impl PipxProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for PipxProvider {
    fn name(&self) -> &str {
        "pipx"
    }

    fn description(&self) -> &str {
        "Install and Run Python Applications in Isolated Environments"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(PipxRuntime::new())]
    }
}
