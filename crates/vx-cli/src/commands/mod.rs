//! CLI command implementations
//!
//! Each command is implemented in its own module for better maintainability.
//! Commands implement the `CommandHandler` trait for unified execution.
//!
//! ## Design Principles (inspired by uv)
//!
//! - **Clear grouping**: Tool management, project management, cache management
//! - **Unified verbs**: add, remove, sync, lock, run
//! - **Subcommand organization**: cache, shell, ext
//! - **No redundancy**: Each command has a single, clear purpose
//!
//! ## Module Organization (RFC 0020 Phase 2)
//!
//! - Complex commands: directories with `mod.rs`, `args.rs`, `handler.rs`
//! - Simple commands: single files
//! - Shared utilities: `common.rs` module

// Core handler trait and context
mod handler;
pub use handler::{CommandContext, CommandHandler, GlobalOptions};

// Shared utilities
pub mod common;

// =============================================================================
// Modular Commands (RFC 0020 Phase 2)
// =============================================================================

/// Global package management - RFC 0025
pub mod global;

/// Install command - modular structure
pub mod install;

/// List command - modular structure
pub mod list;

/// Test command - modular structure (RFC 0020)
pub mod test;

// =============================================================================
// Core Commands
// =============================================================================

pub mod analyze;
pub mod auth;
pub mod bundle;
pub mod cache;
pub mod capabilities;
pub mod check;
pub mod config;
pub mod container;
pub mod dev;
pub mod env;
pub mod execute;
#[cfg(test)]
mod execute_tests;
pub mod ext;
pub mod fetch;
pub mod hook;
pub mod init;
pub mod lock;
pub mod metrics;
pub mod migrate;
pub mod plugin;
pub mod remove;
pub mod run;
pub mod search;
pub mod self_update;
pub mod services;
pub mod setup;
pub mod shell;
pub mod switch;
pub mod sync;
pub mod version;
pub mod where_cmd;

// Re-export vx_env functions for backwards compatibility
pub use vx_env::{execute_with_env, generate_wrapper_script};
