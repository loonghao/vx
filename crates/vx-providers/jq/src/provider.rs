//! jq provider implementation

use crate::runtime::JqRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// jq provider
///
/// Provides the jq JSON processor runtime.
#[derive(Debug, Clone, Default)]
pub struct JqProvider;

impl JqProvider {
    /// Create a new jq provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for JqProvider {
    fn name(&self) -> &str {
        "jq"
    }

    fn description(&self) -> &str {
        "Lightweight and flexible command-line JSON processor"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(JqRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "jq"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "jq" {
            Some(Arc::new(JqRuntime::new()))
        } else {
            None
        }
    }
}
