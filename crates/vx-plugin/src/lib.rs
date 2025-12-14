//! # vx-plugin
//!
//! Plugin system for vx - Universal Development Tool Manager
//!
//! This crate provides the core plugin architecture for vx, enabling developers to create
//! custom tools and package managers that integrate seamlessly with the vx ecosystem.
//!
//! ## Migration Notice
//!
//! The following types have been renamed for clarity:
//! - `VxPlugin` → `ToolBundle` (a bundle of tools and package managers)
//! - `VxPackageManager` → `PackageManager`
//! - `VxTool` remains as `VxTool` (or use `Tool` from vx-sdk)
//!
//! The old names are still available but deprecated.
//!
//! ## Features
//!
//! - **Tool Plugins**: Create custom tool implementations with automatic version management
//! - **Package Manager Plugins**: Integrate custom package managers with unified interfaces
//! - **Plugin Registry**: Discover and manage plugins dynamically
//! - **Extensible Architecture**: Clean trait-based design for maximum flexibility
//!
//! ## Quick Start
//!
//! ### Creating a Simple Tool
//!
//! ```rust,no_run
//! use vx_plugin::{VxTool, VersionInfo, Result};
//! use async_trait::async_trait;
//!
//! struct MyTool;
//!
//! #[async_trait]
//! impl VxTool for MyTool {
//!     fn name(&self) -> &str {
//!         "mytool"
//!     }
//!
//!     async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
//!         // Implementation here
//!         Ok(vec![])
//!     }
//! }
//! ```
//!
//! ### Creating a Tool Bundle (formerly Plugin)
//!
//! ```rust,no_run
//! use vx_plugin::{ToolBundle, VxTool, PackageManager, Result};
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
//!         "A bundle that provides custom tools and package managers"
//!     }
//!
//!     fn tools(&self) -> Vec<Box<dyn VxTool>> {
//!         vec![]
//!     }
//!
//!     fn package_managers(&self) -> Vec<Box<dyn PackageManager>> {
//!         vec![]
//!     }
//! }
//! ```

// Re-export core types and traits for convenience
// New names (preferred)
pub use package_manager::{PackageManager, StandardPackageManager};
pub use plugin::{StandardBundle, ToolBundle};
pub use registry::{BundleRegistry, BundleRegistryBuilder, ToolRegistry};
pub use tool::{ConfigurableTool, UrlBuilder, VersionParser, VxTool};
pub use types::*;

// Deprecated aliases for backward compatibility
#[deprecated(since = "0.5.0", note = "Use `ToolBundle` instead")]
pub use plugin::ToolBundle as VxPlugin;

#[deprecated(since = "0.5.0", note = "Use `PackageManager` instead")]
pub use package_manager::PackageManager as VxPackageManager;

#[deprecated(since = "0.5.0", note = "Use `StandardBundle` instead")]
pub use plugin::StandardBundle as StandardPlugin;

#[deprecated(since = "0.5.0", note = "Use `BundleRegistry` instead")]
pub use registry::BundleRegistry as PluginRegistry;

#[deprecated(since = "0.5.0", note = "Use `BundleRegistryBuilder` instead")]
pub use registry::BundleRegistryBuilder as PluginRegistryBuilder;

// Module declarations
pub mod package_manager;
pub mod plugin;
pub mod registry;
pub mod tool;
pub mod types;

// Utility modules
pub mod utils;

// Result type alias for convenience
pub type Result<T> = anyhow::Result<T>;

/// Plugin system version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Plugin API version for compatibility checking
pub const API_VERSION: &str = "0.2.0";
