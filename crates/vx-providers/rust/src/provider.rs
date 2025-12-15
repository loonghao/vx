//! Rust provider implementation

use crate::runtime::{CargoRuntime, RustcRuntime, RustupRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Rust provider
#[derive(Debug)]
pub struct RustProvider;

impl RustProvider {
    /// Create a new Rust provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for RustProvider {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "Provides Rust toolchain support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(CargoRuntime::new()),
            Arc::new(RustcRuntime::new()),
            Arc::new(RustupRuntime::new()),
        ]
    }
}
