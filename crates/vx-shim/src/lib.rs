//! VX Shim - Cross-platform executable shim for vx tool manager
//!
//! This crate provides a lightweight, cross-platform executable that acts as a proxy
//! to other executables, similar to scoop-better-shimexe but written in Rust for
//! better cross-platform support and modern features.
//!
//! ## Features
//!
//! - **Cross-Platform**: Works on Windows, macOS, and Linux
//! - **Fast Execution**: Minimal overhead with efficient process management
//! - **Signal Handling**: Proper Ctrl+C and signal forwarding to child processes
//! - **Process Management**: Automatic cleanup of child processes
//! - **Flexible Configuration**: Support for both TOML and legacy Scoop formats
//! - **Environment Variables**: Support for custom environment variables
//! - **Working Directory**: Configurable working directory for target executables
//!
//! ## Usage
//!
//! ### As a Library
//!
//! ```rust,no_run
//! use vx_shim::{ShimConfig, Executor};
//!
//! # fn main() -> anyhow::Result<()> {
//! // Load configuration
//! let config = ShimConfig::parse(r#"
//! path = "/bin/echo"
//! args = "hello"
//! "#)?;
//!
//! // Execute with arguments
//! let executor = Executor::new(config);
//! let exit_code = executor.execute(&["world".to_string()])?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Shim Management
//!
//! ```rust
//! use vx_shim::ShimManager;
//! use tempfile::TempDir;
//!
//! # fn main() -> anyhow::Result<()> {
//! let temp_dir = TempDir::new()?;
//! let manager = ShimManager::new(temp_dir.path());
//!
//! // Create a shim
//! let shim_path = manager.create_shim("git", "/usr/bin/git", Some("status"))?;
//!
//! // List shims
//! let shims = manager.list_shims()?;
//!
//! // Remove shim
//! manager.remove_shim("git")?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod executor;
pub mod platform;
pub mod shim;

// Re-export main types for convenience
pub use config::ShimConfig;
pub use executor::Executor;
pub use shim::ShimManager;
