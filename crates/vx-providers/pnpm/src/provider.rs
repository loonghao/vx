//! PNPM provider implementation

use crate::runtime::PnpmRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// PNPM provider
#[derive(Debug)]
pub struct PnpmProvider;

impl PnpmProvider {
    /// Create a new PNPM provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for PnpmProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for PnpmProvider {
    fn name(&self) -> &str {
        "pnpm"
    }

    fn description(&self) -> &str {
        "Provides PNPM package manager support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(PnpmRuntime::new())]
    }
}
