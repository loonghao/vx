//! # vx-plugin
//!
//! Plugin system for vx - Universal Development Tool Manager
//!
//! This crate provides the core plugin architecture for vx, enabling developers to create
//! custom tools and package managers that integrate seamlessly with the vx ecosystem.
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
//! ### Creating a Simple Tool Plugin
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
//! ### Creating a Package Manager Plugin
//!
//! ```rust,no_run//! use vx_plugin::{VxPackageManager, Ecosystem, PackageSpec, Result};
//! use async_trait::async_trait;
//! use std::path::Path;
//!
//! struct MyPackageManager;
//!
//! #[async_trait]
//! impl VxPackageManager for MyPackageManager {
//!     fn name(&self) -> &str {
//!         "mypm"
//!     }
//!
//!     fn ecosystem(&self) -> Ecosystem {
//!         Ecosystem::Node
//!     }
//!
//!     async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
//!         // Implementation here
//!         Ok(())
//!     }
//! }
//! ```

// Re-export core types and traits for convenience
pub use package_manager::{StandardPackageManager, VxPackageManager};
pub use plugin::{StandardPlugin, VxPlugin};
pub use registry::{PluginRegistry, PluginRegistryBuilder, ToolRegistry};
pub use tool::{ConfigurableTool, UrlBuilder, VersionParser, VxTool};
pub use types::*;

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
