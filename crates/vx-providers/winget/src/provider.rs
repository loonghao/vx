//! Windows Package Manager provider implementation

use crate::runtime::WingetRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Windows Package Manager provider
#[derive(Debug)]
pub struct WingetProvider;

impl WingetProvider {
    /// Create a new winget provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for WingetProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for WingetProvider {
    fn name(&self) -> &str {
        // Sourced from provider.star: `def name(): return "winget"`
        crate::star_metadata().name_or("winget")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("winget")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(WingetRuntime::new())]
    }
}
