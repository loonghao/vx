//! Chocolatey provider implementation

use crate::runtime::ChocoRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Chocolatey provider
#[derive(Debug)]
pub struct ChocoProvider;

impl ChocoProvider {
    /// Create a new Chocolatey provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for ChocoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for ChocoProvider {
    fn name(&self) -> &str {
        // Sourced from provider.star: `def name(): return "choco"`
        crate::star_metadata().name_or("choco")
    }

    fn description(&self) -> &str {
        // Sourced from provider.star: `def description(): return "..."`
        use std::sync::OnceLock;
        static DESC: OnceLock<&'static str> = OnceLock::new();
        DESC.get_or_init(|| {
            let s = crate::star_metadata()
                .description
                .as_deref()
                .unwrap_or("Provides Chocolatey package manager support for Windows");
            Box::leak(s.to_string().into_boxed_str())
        })
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ChocoRuntime::new())]
    }
}
