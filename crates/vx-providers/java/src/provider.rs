//! Java provider implementation

use crate::runtime::JavaRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Java provider
#[derive(Debug)]
pub struct JavaProvider;

impl JavaProvider {
    /// Create a new Java provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for JavaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for JavaProvider {
    fn name(&self) -> &str {
        "java"
    }

    fn description(&self) -> &str {
        "Provides Java (Temurin JDK) runtime support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(JavaRuntime::new())]
    }
}
