//! Chocolatey provider implementation

use crate::runtime::ChocoRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Chocolatey provider
#[derive(Debug)]
pub struct ChocoProvider;

impl ChocoProvider {
    /// Create a new Chocolatey provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for ChocoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for ChocoProvider {
    fn name(&self) -> &str {
        // Sourced from provider.star: `def name(): return "choco"`
        crate::star_metadata().name_or("choco")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("choco")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ChocoRuntime::new())]
    }
}
