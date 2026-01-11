//! NASM provider implementation
//!
//! Provides the NASM assembler.

use crate::runtime::NasmRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// NASM provider
#[derive(Debug, Default)]
pub struct NasmProvider;

impl NasmProvider {
    /// Create a new NASM provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for NasmProvider {
    fn name(&self) -> &str {
        "nasm"
    }

    fn description(&self) -> &str {
        "NASM - Netwide Assembler for x86 and x86-64"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(NasmRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "nasm" || name == "ndisasm"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "nasm" || name == "ndisasm" {
            Some(Arc::new(NasmRuntime::new()))
        } else {
            None
        }
    }
}
