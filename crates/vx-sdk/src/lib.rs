//! # vx-sdk
//!
//! Tool Development SDK for vx - Universal Development Tool Manager
//!
//! This crate provides the unified SDK for developing tools and bundles that integrate
//! with the vx ecosystem.
//!
//! ## Features
//!
//! - **Tool Trait**: Core interface for implementing tool support
//! - **ToolBundle Trait**: Group related tools and package managers
//! - **PackageManager Trait**: Unified interface for package managers
//! - **Standard Implementations**: Ready-to-use implementations for common patterns
//! - **Helpers**: URL builders, version utilities, and platform helpers
//!
//! ## Quick Start
//!
//! ### Creating a Simple Tool
//!
//! ```rust,no_run
//! use vx_sdk::{Tool, VersionInfo, Result};
//! use async_trait::async_trait;
//!
//! struct MyTool;
//!
//! #[async_trait]
//! impl Tool for MyTool {
//!     fn name(&self) -> &str {
//!         "mytool"
//!     }
//!
//!     async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
//!         Ok(vec![VersionInfo::new("1.0.0")])
//!     }
//! }
//! ```
//!
//! ### Creating a Tool Bundle
//!
//! ```rust,no_run
//! use vx_sdk::{ToolBundle, Tool, PackageManager, Result};
//! use async_trait::async_trait;
//!
//! struct MyBundle;
//!
//! #[async_trait]
//! impl ToolBundle for MyBundle {
//!     fn name(&self) -> &str {
//!         "my-bundle"
//!     }
//!
//!     fn description(&self) -> &str {
//!         "A bundle providing custom tools"
//!     }
//!
//!     fn tools(&self) -> Vec<Box<dyn Tool>> {
//!         vec![]
//!     }
//!
//!     fn package_managers(&self) -> Vec<Box<dyn PackageManager>> {
//!         vec![]
//!     }
//! }
//! ```

// Re-export async_trait for convenience
pub use async_trait::async_trait;

// Core modules
pub mod helpers;
pub mod registry;
pub mod standard;
pub mod traits;
pub mod types;

// Re-export core traits (new names)
pub use traits::bundle::ToolBundle;
pub use traits::package_manager::PackageManager;
pub use traits::tool::Tool;

// Re-export standard implementations
pub use standard::{ConfigurableTool, StandardBundle, StandardPackageManager};

// Re-export registry
pub use registry::{BundleRegistry, BundleRegistryBuilder, ToolRegistry};

// Re-export types
pub use types::*;

// Re-export helpers
pub use helpers::{PlatformUrlBuilder, UrlUtils, VersionUtils};

// Deprecated aliases for backward compatibility with vx-plugin
#[deprecated(since = "0.5.0", note = "Use `Tool` instead")]
pub type VxTool = dyn Tool;

#[deprecated(since = "0.5.0", note = "Use `ToolBundle` instead")]
pub type VxPlugin = dyn ToolBundle;

#[deprecated(since = "0.5.0", note = "Use `PackageManager` instead")]
pub type VxPackageManager = dyn PackageManager;

#[deprecated(since = "0.5.0", note = "Use `StandardBundle` instead")]
pub type StandardPlugin = StandardBundle;

#[deprecated(since = "0.5.0", note = "Use `BundleRegistry` instead")]
pub type PluginRegistry = BundleRegistry;

#[deprecated(since = "0.5.0", note = "Use `BundleRegistryBuilder` instead")]
pub type PluginRegistryBuilder = BundleRegistryBuilder;

/// Result type alias for convenience
pub type Result<T> = anyhow::Result<T>;

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// API version for compatibility checking
pub const API_VERSION: &str = "0.5.0";
