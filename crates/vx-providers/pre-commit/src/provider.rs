//! pre-commit provider implementation
//!
//! Provides the pre-commit framework.

use crate::runtime::PreCommitRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// pre-commit provider
#[derive(Debug, Default)]
pub struct PreCommitProvider;

impl PreCommitProvider {
    /// Create a new pre-commit provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for PreCommitProvider {
    fn name(&self) -> &str {
        "pre-commit"
    }

    fn description(&self) -> &str {
        "pre-commit - A framework for managing and maintaining multi-language pre-commit hooks"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(PreCommitRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "pre-commit"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "pre-commit" {
            Some(Arc::new(PreCommitRuntime::new()))
        } else {
            None
        }
    }
}
