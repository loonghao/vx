//! Conda provider implementation
//!
//! Bundles Micromamba, Conda, and Mamba runtimes.

use crate::runtime::{CondaRuntime, MambaRuntime, MicromambaRuntime};
use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// Conda provider that bundles Conda package management tools
///
/// This provider includes:
/// - `micromamba` - Minimal standalone mamba (recommended, single binary)
/// - `conda` - Package and environment manager (via Miniforge)
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
            Arc::new(MicromambaRuntime::new()),
            Arc::new(CondaRuntime::new()),
            Arc::new(MambaRuntime::new()),
        ]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(
            name,
            "micromamba" | "umamba" | "conda" | "miniforge" | "miniconda" | "mamba"
        )
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        match name {
            "micromamba" | "umamba" => Some(Arc::new(MicromambaRuntime::new())),
            "conda" | "miniforge" | "miniconda" => Some(Arc::new(CondaRuntime::new())),
            "mamba" => Some(Arc::new(MambaRuntime::new())),
            _ => None,
        }
    }
}

/// Create a new Conda provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(CondaProvider::new())
}
