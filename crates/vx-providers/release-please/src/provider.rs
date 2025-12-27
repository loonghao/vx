//! release-please provider implementation
//!
//! Provides the release-please CLI tool.

use crate::runtime::ReleasePleaseRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// release-please provider
#[derive(Debug, Default)]
pub struct ReleasePleaseProvider;

impl ReleasePleaseProvider {
    /// Create a new release-please provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for ReleasePleaseProvider {
    fn name(&self) -> &str {
        "release-please"
    }

    fn description(&self) -> &str {
        "release-please - Automate releases based on conventional commits"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ReleasePleaseRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "release-please"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "release-please" {
            Some(Arc::new(ReleasePleaseRuntime::new()))
        } else {
            None
        }
    }
}
