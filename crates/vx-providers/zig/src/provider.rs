//! Zig provider implementation

use crate::runtime::ZigRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Zig provider
#[derive(Debug)]
pub struct ZigProvider;

impl ZigProvider {
    /// Create a new Zig provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for ZigProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for ZigProvider {
    fn name(&self) -> &str {
        "zig"
    }

    fn description(&self) -> &str {
        "Provides Zig programming language support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ZigRuntime::new())]
    }
}
