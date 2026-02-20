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
        // Sourced from provider.star: `def description(): return "..."`
        use std::sync::OnceLock;
        static DESC: OnceLock<&'static str> = OnceLock::new();
        DESC.get_or_init(|| {
            let s = crate::star_metadata()
                .description
                .as_deref()
                .unwrap_or("Provides Homebrew package manager support for macOS and Linux");
            Box::leak(s.to_string().into_boxed_str())
        })
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(BrewRuntime::new())]
    }
}
