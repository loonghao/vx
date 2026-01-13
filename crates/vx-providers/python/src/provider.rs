//! Python provider implementation

use crate::runtime::{PipRuntime, PythonRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Python provider for vx
///
/// Provides Python runtime using python-build-standalone distributions
/// from Astral (https://github.com/astral-sh/python-build-standalone)
#[derive(Debug, Default)]
pub struct PythonProvider;

impl PythonProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for PythonProvider {
    fn name(&self) -> &str {
        "python"
    }

    fn description(&self) -> &str {
        "Python programming language (using python-build-standalone)"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(PythonRuntime::new()), Arc::new(PipRuntime::new())]
    }
}
