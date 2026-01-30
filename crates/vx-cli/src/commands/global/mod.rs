//! Global package management commands (RFC 0025)
//!
//! This module provides commands for managing globally installed packages
//! using vx's isolated global package system.
//!
//! Commands:
//! - `vx install-global` - Install a package globally
//! - `vx list-global` - List globally installed packages
//! - `vx uninstall-global` - Remove a global package
//! - `vx info-global` - Show information about a global package

mod args;
mod handler;

pub use args::{
    GlobalCommand, InfoGlobalArgs, InstallGlobalArgs, ListGlobalArgs, UninstallGlobalArgs,
};
pub use handler::handle;
