//! Deno provider implementation

use crate::runtime::DenoRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Deno provider
#[derive(Debug)]
pub struct DenoProvider;

impl DenoProvider {
    /// Create a new Deno provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for DenoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for DenoProvider {
    fn name(&self) -> &str {
        "deno"
    }

    fn description(&self) -> &str {
        "Provides Deno JavaScript/TypeScript runtime support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(DenoRuntime::new())]
    }
}
