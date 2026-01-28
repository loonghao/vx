//! Just provider implementation
//!
//! Provides the Just command runner.

use crate::runtime::JustRuntime;
use std::sync::Arc;
use vx_runtime::{provider::Provider, Runtime};

/// Just provider
#[derive(Debug, Default)]
pub struct JustProvider;

impl JustProvider {
    /// Create a new Just provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for JustProvider {
    fn name(&self) -> &str {
        "just"
    }

    fn description(&self) -> &str {
        "Just - A handy way to save and run project-specific commands"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(JustRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "just"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "just" {
            Some(Arc::new(JustRuntime::new()))
        } else {
            None
        }
    }
}

/// Create the Just provider
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JustProvider::new())
}
