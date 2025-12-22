//! rcedit provider implementation
//!
//! Provides the rcedit Windows resource editor tool.

use crate::runtime::RceditRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// rcedit provider
#[derive(Debug, Default)]
pub struct RceditProvider;

impl RceditProvider {
    /// Create a new rcedit provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for RceditProvider {
    fn name(&self) -> &str {
        "rcedit"
    }

    fn description(&self) -> &str {
        "rcedit - Command-line tool to edit resources of Windows executables"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(RceditRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "rcedit"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "rcedit" {
            Some(Arc::new(RceditRuntime::new()))
        } else {
            None
        }
    }
}
