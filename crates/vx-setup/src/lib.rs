//! VX Setup Pipeline and CI Environment Support
//!
//! This crate provides the setup pipeline functionality for `vx setup`.
//! The setup process is a pipeline of hooks that can be customized.
//!
//! # Features
//!
//! - **CI Environment Detection**: Automatically detect CI providers (GitHub Actions, GitLab CI, etc.)
//! - **Path Export**: Export tool paths to CI environment variables
//! - **Setup Pipeline**: Configurable pipeline of hooks for setup process
//!
//! # Default Pipeline
//!
//! 1. `pre_setup` - User-defined hook (from [hooks] section)
//! 2. `install_tools` - Built-in: Install all configured tools
//! 3. `export_paths` - Built-in: Export tool paths for CI environments
//! 4. `post_setup` - User-defined hook (from [hooks] section)
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_setup::{SetupPipeline, SetupPipelineConfig};
//! use vx_setup::ci::CiProvider;
//!
//! // Create a setup pipeline
//! let pipeline = SetupPipeline::new(".", "~/.vx/store", "~/.vx/bin")
//!     .with_config(config)
//!     .force_ci(true);
//!
//! // Execute the pipeline
//! let result = pipeline.execute(|| async {
//!     // Install tools here
//!     Ok(())
//! }).await?;
//!
//! if result.success {
//!     println!("Setup completed successfully!");
//! }
//! ```

pub mod ci;
mod error;
mod pipeline;
mod types;

pub use ci::CiProvider;
pub use error::{SetupError, SetupResult};
pub use pipeline::{SetupHookResult, SetupPipeline, SetupPipelineResult};
pub use types::{HookCommand, SetupPipelineConfig};
