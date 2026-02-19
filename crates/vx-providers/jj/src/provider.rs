//! Jujutsu (jj) provider implementation
//!
//! Provides the jj Git-compatible DVCS tool.

use crate::runtime::JjRuntime;
use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// jj provider
#[derive(Debug, Default)]
pub struct JjProvider;

impl JjProvider {
    /// Create a new jj provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for JjProvider {
    fn name(&self) -> &str {
        "jj"
    }

    fn description(&self) -> &str {
        "Jujutsu (jj) - A Git-compatible DVCS that is both simple and powerful"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(JjRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "jj"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "jj" {
            Some(Arc::new(JjRuntime::new()))
        } else {
            None
        }
    }
}

/// Create the jj provider
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JjProvider::new())
}
