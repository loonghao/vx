//! Homebrew provider implementation

use crate::runtime::BrewRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Homebrew provider
#[derive(Debug)]
pub struct BrewProvider;

impl BrewProvider {
    /// Create a new Homebrew provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for BrewProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for BrewProvider {
    fn name(&self) -> &str {
        "brew"
    }

    fn description(&self) -> &str {
        "Provides Homebrew package manager support for macOS and Linux"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(BrewRuntime::new())]
    }
}
