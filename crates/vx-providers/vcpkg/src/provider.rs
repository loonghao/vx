//! vcpkg provider implementation

use crate::runtime::VcpkgRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// vcpkg provider
pub struct VcpkgProvider {
    runtimes: Vec<Arc<dyn Runtime>>,
}

impl VcpkgProvider {
    /// Create a new vcpkg provider
    pub fn new() -> Self {
        let runtimes: Vec<Arc<dyn Runtime>> = vec![Arc::new(VcpkgRuntime::new())];
        Self { runtimes }
    }
}

impl Default for VcpkgProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for VcpkgProvider {
    fn name(&self) -> &str {
        "vcpkg"
    }

    fn description(&self) -> &str {
        "vcpkg - C++ library manager for Windows, Linux, and macOS"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        self.runtimes.clone()
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "vcpkg")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if self.supports(name) {
            self.runtimes.first().cloned()
        } else {
            None
        }
    }
}
