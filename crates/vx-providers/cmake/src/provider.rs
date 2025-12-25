//! CMake provider implementation
//!
//! Provides the CMake build system.

use crate::runtime::CMakeRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// CMake provider
#[derive(Debug, Default)]
pub struct CMakeProvider;

impl CMakeProvider {
    /// Create a new CMake provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for CMakeProvider {
    fn name(&self) -> &str {
        "cmake"
    }

    fn description(&self) -> &str {
        "CMake - Cross-platform build system generator"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(CMakeRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "cmake" | "ctest" | "cpack")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if matches!(name, "cmake" | "ctest" | "cpack") {
            Some(Arc::new(CMakeRuntime::new()))
        } else {
            None
        }
    }
}
