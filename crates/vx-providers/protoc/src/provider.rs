//! Protoc provider implementation
//!
//! Provides the Protocol Buffers compiler.

use crate::runtime::ProtocRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Protoc provider
#[derive(Debug, Default)]
pub struct ProtocProvider;

impl ProtocProvider {
    /// Create a new protoc provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for ProtocProvider {
    fn name(&self) -> &str {
        "protoc"
    }

    fn description(&self) -> &str {
        "Protocol Buffers Compiler - Google's data interchange format"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ProtocRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "protoc" || name == "protobuf"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "protoc" || name == "protobuf" {
            Some(Arc::new(ProtocRuntime::new()))
        } else {
            None
        }
    }
}
