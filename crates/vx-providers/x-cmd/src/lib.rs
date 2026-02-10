//! x-cmd command-line toolbox provider for vx
//!
//! This crate provides x-cmd support using the vx-runtime traits.
//! x-cmd is a compact and powerful command-line toolbox with 100+ built-in modules
//! and a package manager for 500+ third-party CLI tools.
//!
//! ## Features
//!
//! - AI integration (chat, agent, code generation)
//! - 100+ built-in modules
//! - Package manager for 500+ CLI tools
//! - Environment management for Node, Python, Java, Go
//! - Cross-platform: Linux, macOS, Windows

mod config;
mod provider;
mod runtime;

pub use config::XCmdConfig;
pub use provider::XCmdProvider;
pub use runtime::XCmdRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new x-cmd provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(XCmdProvider::new())
}
