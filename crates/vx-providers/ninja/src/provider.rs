//! Ninja provider implementation
//!
//! Provides the Ninja build system.

use crate::runtime::NinjaRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Ninja provider
#[derive(Debug, Default)]
pub struct NinjaProvider;

impl NinjaProvider {
    /// Create a new Ninja provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for NinjaProvider {
    fn name(&self) -> &str {
        "ninja"
    }

    fn description(&self) -> &str {
        "Ninja - A small build system with a focus on speed"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(NinjaRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "ninja"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "ninja" {
            Some(Arc::new(NinjaRuntime::new()))
        } else {
            None
        }
    }
}
