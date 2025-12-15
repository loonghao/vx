//! Yarn provider implementation

use crate::runtime::YarnRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Yarn provider
#[derive(Debug)]
pub struct YarnProvider;

impl YarnProvider {
    /// Create a new Yarn provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for YarnProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for YarnProvider {
    fn name(&self) -> &str {
        "yarn"
    }

    fn description(&self) -> &str {
        "Provides Yarn package manager support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(YarnRuntime::new())]
    }
}
