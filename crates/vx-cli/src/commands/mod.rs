//! CLI command implementations
//!
//! Each command is implemented in its own module for better maintainability.
//! Commands implement the `CommandHandler` trait for unified execution.
//!
//! RFC 0020 Phase 2: Modular command structure
//! - Commands with complex args are organized as directories (mod.rs, args.rs, handler.rs)
//! - Simple commands remain as single files
//! - All commands export Args and handle() for unified interface

// Core handler trait and context
mod handler;
pub use handler::{CommandContext, CommandHandler, GlobalOptions};

// =============================================================================
// Modular Commands (RFC 0020 Phase 2)
// =============================================================================
// These commands are organized as directories with args.rs and handler.rs

/// Install command - modular structure
pub mod install;

/// List command - modular structure
pub mod list;

/// Test command - modular structure (RFC 0020)
pub mod test;

// =============================================================================
// Simple Commands
// =============================================================================
// These commands remain as single files

pub mod analyze;
pub mod cache;
pub mod capabilities;
pub mod check;
pub mod cleanup;
pub mod config;
pub mod container;
pub mod dev;
pub mod env;
pub mod execute;
#[cfg(test)]
mod execute_tests;
pub mod ext;
pub mod fetch;
pub mod global;
pub mod hook;
pub mod init;
pub mod lock;
pub mod migrate;
pub mod plugin;
pub mod remove;
pub mod run;
pub mod search;
pub mod self_update;
pub mod services;
pub mod setup;
pub mod shell;
pub mod stats;
pub mod switch;
pub mod sync;
pub mod update;
pub mod venv_cmd;
pub mod version;
pub mod where_cmd;

// Re-export vx_env functions for backwards compatibility
pub use vx_env::{execute_with_env, generate_wrapper_script};
