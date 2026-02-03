//! Executor Module
//!
//! This module implements the main execution logic for runtime command forwarding:
//! 1. Resolve runtime and dependencies
//! 2. Auto-install missing components  
//! 3. Forward command to the appropriate executable
//!
//! ## Module Structure
//!
//! - `core` - The main Executor struct and execution logic
//! - `project_config` - Project configuration loading from vx.toml
//! - `bundle` - Offline bundle support for disconnected environments

mod bundle;
mod core;
mod project_config;

// Re-export main types
pub use bundle::{
    execute_bundle, execute_system_runtime, has_bundle, is_online, try_get_bundle_context,
    BundleContext, BundleManifest, BundledToolInfo, BUNDLE_DIR, BUNDLE_MANIFEST,
};
pub use core::Executor;
pub use project_config::ProjectToolsConfig;

// Re-export from vx_core for convenience
pub use vx_core::{exit_code_from_status, is_ctrl_c_exit};
