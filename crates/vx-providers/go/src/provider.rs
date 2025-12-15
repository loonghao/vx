//! Go provider implementation
//!
//! This module provides the GoProvider which bundles the Go runtime.

use crate::runtime::GoRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Go provider that provides the Go programming language runtime
#[derive(Debug, Default)]
pub struct GoProvider;

impl GoProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for GoProvider {
    fn name(&self) -> &str {
        "go"
    }

    fn description(&self) -> &str {
        "Go programming language support for vx"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(GoRuntime::new())]
    }
}
