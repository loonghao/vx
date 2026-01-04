//! MSVC Build Tools provider implementation
//!
//! Provides the MSVC Build Tools runtime for C/C++ compilation on Windows.
//! Downloads directly from Microsoft's official servers.

use crate::runtime::MsvcRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// MSVC Build Tools provider
#[derive(Debug, Default)]
pub struct MsvcProvider;

impl MsvcProvider {
    /// Create a new MSVC provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for MsvcProvider {
    fn name(&self) -> &str {
        "msvc"
    }

    fn description(&self) -> &str {
        "MSVC Build Tools - Microsoft Visual C++ compiler and tools (downloads from Microsoft)"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(MsvcRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "msvc" | "cl" | "msvc-tools" | "vs-build-tools")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if self.supports(name) {
            Some(Arc::new(MsvcRuntime::new()))
        } else {
            None
        }
    }
}
