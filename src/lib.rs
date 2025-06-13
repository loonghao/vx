//! VX - Universal Development Tool Manager
//!
//! This is the root library crate that re-exports the main CLI functionality
//! and provides integration test support.

pub use vx_cli::*;

// Re-export core types for integration tests
pub use vx_core::{
    Environment, PluginRegistry, VenvManager, VxConfig, VxPackageManager, VxPlugin, VxTool,
};

// Re-export plugin implementations for integration tests
pub use vx_tool_go;
pub use vx_tool_node;
pub use vx_tool_rust;
pub use vx_tool_uv;
