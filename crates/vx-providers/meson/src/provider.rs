//! Meson provider implementation

use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

use crate::MesonRuntime;

/// Meson provider
#[derive(Debug, Default)]
pub struct MesonProvider;

impl MesonProvider {
    /// Create a new Meson provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for MesonProvider {
    fn name(&self) -> &str {
        "meson"
    }

    fn description(&self) -> &str {
        "Meson build system - extremely fast and user friendly"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(MesonRuntime::new())]
    }
}
