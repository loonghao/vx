//! Conda provider implementation
//!
//! This module provides the CondaProvider which bundles Micromamba, Conda, and Mamba runtimes.
//!
//! Micromamba is listed first as it's the recommended choice for vx users.

use crate::runtime::{CondaRuntime, MambaRuntime, MicromambaRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Conda provider that bundles Conda package management tools
///
/// This provider includes:
/// - `micromamba` - Minimal standalone mamba (recommended, single binary)
/// - `conda` - Package and environment manager (via Miniforge, requires installer)
/// - `mamba` - Fast package manager (bundled with Miniforge)
#[derive(Debug, Default)]
pub struct CondaProvider;

impl CondaProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for CondaProvider {
    fn name(&self) -> &str {
        "conda"
    }

    fn description(&self) -> &str {
        "Conda package, dependency and environment management"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            // Micromamba first - it's the recommended choice
            Arc::new(MicromambaRuntime::new()),
            Arc::new(CondaRuntime::new()),
            Arc::new(MambaRuntime::new()),
        ]
    }
}
