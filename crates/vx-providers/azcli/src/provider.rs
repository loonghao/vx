//! Azure CLI provider implementation

use crate::runtime::AzCliRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Azure CLI provider
#[derive(Debug)]
pub struct AzCliProvider;

impl AzCliProvider {
    /// Create a new Azure CLI provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for AzCliProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for AzCliProvider {
    fn name(&self) -> &str {
        "azcli"
    }

    fn description(&self) -> &str {
        "Provides Azure CLI support for Microsoft Azure"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(AzCliRuntime::new())]
    }
}
