//! Environment command implementation
//!
//! Modular command structure following RFC 0020 Phase 2.
//!
//! This module provides commands for managing vx environments:
//! - create: Create a new environment (project-local or global)
//! - use: Activate an environment
//! - list: List all environments
//! - delete: Remove an environment
//! - show: Show current environment details
//! - shell: Enter an interactive shell with environment tools
//!
//! ## Environment Types
//!
//! - **Project Environment**: Created in `.vx/env/` under the project directory
//! - **Global Environment**: Created in `~/.vx/envs/` for cross-project use
//!
//! ## Storage Model
//!
//! All tools are stored globally in `~/.vx/store/` (content-addressable).
//! Environments contain symlinks to the global store, saving disk space.

mod args;
mod handler;
mod helpers;

pub use args::{Args, EnvCommand};
pub use handler::handle;
