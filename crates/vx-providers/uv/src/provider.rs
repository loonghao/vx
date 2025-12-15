//! UV provider implementation
//!
//! This module provides the UvProvider which bundles UV and UVX runtimes.

use crate::runtime::{UvRuntime, UvxRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// UV provider that bundles UV Python package management tools
///
/// This provider includes:
/// - `uv` - Python package installer and resolver
/// - `uvx` - Python application runner
#[derive(Debug, Default)]
pub struct UvProvider;

impl UvProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for UvProvider {
    fn name(&self) -> &str {
        "uv"
    }

    fn description(&self) -> &str {
        "UV Python package management tools"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(UvRuntime::new()), Arc::new(UvxRuntime::new())]
    }
}
