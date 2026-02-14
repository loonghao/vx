//! Hadolint provider implementation
//!
//! Provides the Hadolint (Dockerfile linter) runtime.

use crate::runtime::HadolintRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Hadolint provider
#[derive(Debug)]
pub struct HadolintProvider;

impl HadolintProvider {
    /// Create a new Hadolint provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for HadolintProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for HadolintProvider {
    fn name(&self) -> &str {
        "hadolint"
    }

    fn description(&self) -> &str {
        "Provides Hadolint (Dockerfile linter) support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(HadolintRuntime::new())]
    }
}
