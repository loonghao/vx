//! fd provider implementation

use crate::runtime::FdRuntime;
use std::sync::Arc;
use vx_runtime::{provider::Provider, Runtime};

/// fd provider
#[derive(Debug, Default)]
pub struct FdProvider;

impl FdProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for FdProvider {
    fn name(&self) -> &str {
        "fd"
    }

    fn description(&self) -> &str {
        "A simple, fast and user-friendly alternative to find"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(FdRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "fd" | "fd-find")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if matches!(name, "fd" | "fd-find") {
            Some(Arc::new(FdRuntime::new()))
        } else {
            None
        }
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FdProvider::new())
}
