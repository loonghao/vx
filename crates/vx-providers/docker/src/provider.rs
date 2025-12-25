//! Docker provider implementation

use crate::runtime::DockerRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Docker provider
#[derive(Debug)]
pub struct DockerProvider;

impl DockerProvider {
    /// Create a new Docker provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for DockerProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for DockerProvider {
    fn name(&self) -> &str {
        "docker"
    }

    fn description(&self) -> &str {
        "Provides Docker CLI support for container management"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(DockerRuntime::new())]
    }
}
