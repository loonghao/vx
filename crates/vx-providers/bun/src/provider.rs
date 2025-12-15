//! Bun provider implementation

use crate::runtime::{BunRuntime, BunxRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Bun provider
#[derive(Debug)]
pub struct BunProvider;

impl BunProvider {
    /// Create a new Bun provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for BunProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for BunProvider {
    fn name(&self) -> &str {
        "bun"
    }

    fn description(&self) -> &str {
        "Provides Bun JavaScript runtime support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(BunRuntime::new()), Arc::new(BunxRuntime::new())]
    }
}
