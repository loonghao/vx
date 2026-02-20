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
        // Sourced from provider.star: `def name(): return "brew"`
        crate::star_metadata().name_or("brew")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("brew")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(BrewRuntime::new())]
    }
}
