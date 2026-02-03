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
        "winget"
    }

    fn description(&self) -> &str {
        "Provides Windows Package Manager (winget) support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(WingetRuntime::new())]
    }
}
