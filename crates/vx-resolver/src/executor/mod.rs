//! Executor Module
//!
//! This module implements the main execution logic for runtime command forwarding:
//! 1. Resolve runtime and dependencies
//! 2. Auto-install missing components  
//! 3. Forward command to the appropriate executable
//!
//! ## Module Structure (Refactored for Single Responsibility)
//!
//! - `executor` - The main Executor struct and execution flow
//! - `installation` - Runtime installation logic
//! - `fallback` - Fallback installation methods (system installers)
//! - `environment` - Environment variable preparation and PATH building
//! - `command` - Command building and execution
//! - `project_config` - Project configuration loading from vx.toml
//! - `bundle` - Offline bundle support for disconnected environments

mod bundle;
mod command;
mod environment;
#[allow(clippy::module_inception)]
mod executor;
mod fallback;
mod installation;
pub mod pipeline;
mod project_config;

// Re-export main types
pub use bundle::{
    execute_bundle, execute_system_runtime, has_bundle, is_online, try_get_bundle_context,
    BundleContext, BundleManifest, BundledToolInfo, BUNDLE_DIR, BUNDLE_MANIFEST,
};
pub use environment::{
    clear_bin_dir_cache, init_bin_dir_cache, invalidate_bin_dir_cache, save_bin_dir_cache,
};
pub use executor::Executor;
pub use project_config::ProjectToolsConfig;

// Re-export from vx_core for convenience
pub use vx_core::{exit_code_from_status, is_ctrl_c_exit};
