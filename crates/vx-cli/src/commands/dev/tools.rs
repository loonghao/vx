//! Tool discovery and version detection utilities
//!
//! This module re-exports tool status checking functions from common.rs
//! for backward compatibility.

use anyhow::Result;
use std::path::PathBuf;
use vx_paths::PathManager;
use vx_runtime::{create_runtime_context, ProviderRegistry};

// Re-export types and functions from common.rs
pub use crate::commands::common::{find_system_tool, get_vx_tool_path, ToolStatus};

/// Get the status and path of a tool (re-exported from common.rs)
///
/// This function is now a thin wrapper around common::check_tool_status()
/// for backward compatibility.
pub fn get_tool_status(
    path_manager: &PathManager,
    tool: &str,
    version: &str,
) -> Result<(ToolStatus, Option<PathBuf>, Option<String>)> {
    crate::commands::common::check_tool_status(path_manager, tool, version)
}

/// Get provider registry and runtime context for dev command
///
/// This function creates a new registry instance for tool installation operations.
/// It's used by dev command to delegate to sync for tool installation.
pub fn get_registry() -> Result<(ProviderRegistry, vx_runtime::RuntimeContext)> {
    let registry = crate::registry::create_registry();
    let context = create_runtime_context()?;
    Ok((registry, context))
}
