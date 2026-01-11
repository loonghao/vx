//! YASM provider implementation
//!
//! Provides the YASM modular assembler.

use crate::runtime::YasmRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// YASM provider
#[derive(Debug, Default)]
pub struct YasmProvider;

impl YasmProvider {
    /// Create a new YASM provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for YasmProvider {
    fn name(&self) -> &str {
        "yasm"
    }

    fn description(&self) -> &str {
        "YASM - Modular Assembler with multiple output formats"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(YasmRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "yasm"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "yasm" {
            Some(Arc::new(YasmRuntime::new()))
        } else {
            None
        }
    }
}
