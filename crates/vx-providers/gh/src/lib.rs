//! GitHub CLI Provider for vx
//!
//! This provider adds support for GitHub CLI (gh) tool to vx.
//!
//! GitHub CLI is a command-line tool that brings GitHub to your terminal.
//!
//! ## Features
//!
//! - Version management via GitHub releases
//! - Cross-platform support (Windows, macOS, Linux)
//! - Automatic installation and verification

use std::sync::Arc;

pub mod config;
pub mod provider;
pub mod runtime;

pub use config::GitHubUrlBuilder;
pub use provider::GitHubProvider;
pub use runtime::GitHubRuntime;

/// Factory function to create the GitHub provider
pub fn create_provider() -> Arc<dyn vx_runtime::Provider> {
    Arc::new(provider::GitHubProvider::new())
}
