//! Helm provider implementation

use crate::runtime::HelmRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Helm provider
#[derive(Debug)]
pub struct HelmProvider;

impl HelmProvider {
    /// Create a new Helm provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for HelmProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for HelmProvider {
    fn name(&self) -> &str {
        "helm"
    }

    fn description(&self) -> &str {
        "Provides Helm (Kubernetes package manager) support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(HelmRuntime::new())]
    }
}
