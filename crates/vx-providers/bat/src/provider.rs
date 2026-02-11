//! bat provider implementation

use crate::runtime::BatRuntime;
use std::sync::Arc;
use vx_runtime::{provider::Provider, Runtime};

/// bat provider
#[derive(Debug, Default)]
pub struct BatProvider;

impl BatProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for BatProvider {
    fn name(&self) -> &str {
        "bat"
    }

    fn description(&self) -> &str {
        "A cat clone with syntax highlighting and Git integration"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(BatRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "bat"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "bat" {
            Some(Arc::new(BatRuntime::new()))
        } else {
            None
        }
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(BatProvider::new())
}
