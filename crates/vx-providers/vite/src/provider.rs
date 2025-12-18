//! Vite provider implementation
//!
//! Provides the Vite frontend build tool.

use crate::runtime::ViteRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Vite provider
#[derive(Debug, Default)]
pub struct ViteProvider;

impl ViteProvider {
    /// Create a new Vite provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for ViteProvider {
    fn name(&self) -> &str {
        "vite"
    }

    fn description(&self) -> &str {
        "Vite - Next Generation Frontend Tooling"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ViteRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "vite"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "vite" {
            Some(Arc::new(ViteRuntime::new()))
        } else {
            None
        }
    }
}
