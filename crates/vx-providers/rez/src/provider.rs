//! Rez provider implementation

use crate::RezRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Rez provider
pub struct RezProvider;

impl RezProvider {
    /// Create a new Rez provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for RezProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for RezProvider {
    fn name(&self) -> &str {
        "rez"
    }

    fn description(&self) -> &str {
        "Rez - Cross-platform package manager for deterministic environments"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(RezRuntime::new())]
    }

    fn package_managers(&self) -> Vec<Arc<dyn vx_runtime::provider::PackageManager>> {
        vec![]
    }

    fn supports(&self, name: &str) -> bool {
        name == "rez" || name == "rez-env" || name == "rez-build" || name == "rez-release"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if self.supports(name) {
            Some(Arc::new(RezRuntime::new()))
        } else {
            None
        }
    }
}
