//! CLI command implementations
//!
//! Each command is implemented in its own module for better maintainability.
//! Commands implement the `CommandHandler` trait for unified execution.

// Core handler trait and context
mod handler;
pub use handler::{CommandContext, CommandHandler, GlobalOptions};

// Command modules
pub mod analyze;
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
pub mod install;
pub mod list;
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
