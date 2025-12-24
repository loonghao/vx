//! Git provider implementation.

use std::sync::Arc;

use vx_runtime::{Provider, Runtime};

use crate::runtime::GitRuntime;

/// Git provider for vx.
#[derive(Debug, Default)]
pub struct GitProvider;

impl GitProvider {
    /// Create a new Git provider instance.
    pub fn new() -> Self {
        Self
    }
}

impl Provider for GitProvider {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "Git version control system support for vx"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(GitRuntime::new())]
    }
}
