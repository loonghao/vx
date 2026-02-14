//! ripgrep provider implementation

use crate::runtime::RipgrepRuntime;
use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// ripgrep provider
#[derive(Debug, Default)]
pub struct RipgrepProvider;

impl RipgrepProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for RipgrepProvider {
    fn name(&self) -> &str {
        "ripgrep"
    }

    fn description(&self) -> &str {
        "ripgrep recursively searches directories for a regex pattern"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(RipgrepRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "rg" | "ripgrep")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if matches!(name, "rg" | "ripgrep") {
            Some(Arc::new(RipgrepRuntime::new()))
        } else {
            None
        }
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RipgrepProvider::new())
}
