//! # VX Configuration Management
//!
//! This crate provides comprehensive configuration management for the vx tool manager.
//! It supports layered configuration from multiple sources and automatic project detection.
//!
//! ## Features
//!
//! - **Layered Configuration**: Supports builtin defaults, user config, project config, and environment variables
//! - **Project Detection**: Automatically detects Python, Rust, Node.js, and Go projects
//! - **Multiple Formats**: Supports TOML, JSON, and other configuration formats
//! - **Tool Version Management**: Manages tool versions across different project types
//!
//! ## Example
//!
//! ```rust
//! use vx_config::ConfigManager;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config_manager = ConfigManager::new().await?;
//! let tool_version = config_manager.get_tool_version("node");
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod detection;
pub mod error;
pub mod manager;
pub mod parsers;
pub mod types;

// Re-export main types and functions
pub use config::*;
pub use error::{ConfigError, Result};
pub use manager::ConfigManager;
pub use types::*;

/// Current version of the vx-config crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
