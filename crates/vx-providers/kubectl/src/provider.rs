//! kubectl provider implementation

use crate::runtime::KubectlRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// kubectl provider
#[derive(Debug)]
pub struct KubectlProvider;

impl KubectlProvider {
    /// Create a new kubectl provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for KubectlProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for KubectlProvider {
    fn name(&self) -> &str {
        "kubectl"
    }

    fn description(&self) -> &str {
        "Provides kubectl (Kubernetes CLI) support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(KubectlRuntime::new())]
    }
}
