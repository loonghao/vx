//! # vx-version
//!
//! Version management and parsing utilities for the vx universal tool manager.
//!
//! This crate provides comprehensive version management capabilities including:
//! - Semantic version parsing and comparison
//! - Version fetching from various sources (GitHub, npm, PyPI, etc.)
//! - Version constraint resolution
//! - Tool-specific version parsing
//!
//! ## Features
//!
//! - **Version Parsing**: Parse and compare semantic versions
//! - **Version Fetching**: Fetch available versions from external sources
//! - **Version Constraints**: Support for version ranges and constraints
//! - **Tool Integration**: Specialized parsers for different tools
//! - **Async Support**: Async-first design for network operations
//!
//! ## Example
//!
//! ```rust
//! use vx_version::{VersionInfo, VersionFetcher, GitHubVersionFetcher};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create a version fetcher for a GitHub repository
//!     let fetcher = GitHubVersionFetcher::new("astral-sh", "uv");
//!     
//!     // Fetch available versions
//!     let versions = fetcher.fetch_versions(false).await?;
//!     
//!     // Get the latest version
//!     if let Some(latest) = versions.first() {
//!         println!("Latest version: {}", latest.version);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod fetcher;
pub mod manager;
pub mod parser;
pub mod utils;

// Re-export main types for convenience
pub use error::{Result, VersionError};
pub use fetcher::{
    CachedVersionFetcher, GitHubVersionFetcher, GoVersionFetcher, NodeVersionFetcher,
    TurboCdnVersionFetcher, VersionFetcher,
};
pub use manager::VersionManager;
pub use parser::{GitHubVersionParser, GoVersionParser, NodeVersionParser, VersionParser};
pub use utils::VersionUtils;

// Re-export VersionInfo from vx-plugin to avoid duplication
pub use vx_plugin::types::VersionInfo;

/// Version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
