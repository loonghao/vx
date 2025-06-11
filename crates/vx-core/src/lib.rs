//! Core traits and interfaces for vx tool manager
//!
//! This crate provides a highly abstracted plugin system that makes it easy for developers
//! to implement new tools and package managers with minimal boilerplate code.
//!
//! # Quick Start for Plugin Developers
//!
//! To create a new tool plugin, implement the `VxTool` trait:
//!
//! ```rust
//! use vx_core::{VxTool, ToolContext, VersionInfo, Result};
//!
//! #[derive(Default)]
//! struct MyTool;
//!
//! #[async_trait::async_trait]
//! impl VxTool for MyTool {
//!     fn name(&self) -> &str { "mytool" }
//!
//!     async fn fetch_versions(&self, _include_prerelease: bool) -> Result<Vec<VersionInfo>> {
//!         // Fetch versions from your tool's API
//!         Ok(vec![])
//!     }
//!
//!     async fn install_version(&self, version: &str, install_dir: &std::path::Path) -> Result<std::path::PathBuf> {
//!         // Download and install the tool
//!         Ok(install_dir.join("bin").join("mytool"))
//!     }
//! }
//! ```

// pub mod plugin; // Temporarily disabled during migration
pub mod tool;
pub mod package_manager;
pub mod version;
pub mod installer;
pub mod install_configs;
pub mod config;
pub mod config_figment;
pub mod error;
// pub mod registry; // Temporarily disabled during migration

// Utility modules
pub mod platform;
pub mod http;
pub mod url_builder;
pub mod version_parser;
pub mod version_manager;
pub mod venv;

// Re-export main traits for convenience
// Temporarily disabled old plugin system during migration
// pub use plugin::{VxTool, VxPackageManager, VxPlugin, UrlBuilder, VersionParser, ConfigurableTool, ToolMetadata, StandardPlugin};
pub use tool::{Tool, AsyncTool, Environment, Configuration, Plugin, ToolContext, ToolExecutionResult, ToolInfo, ToolStatus};
pub use version::VersionInfo;
pub use installer::{InstallConfig, InstallProgress, InstallStage};
pub use install_configs::{get_install_config, supports_auto_install, get_manual_install_instructions};
pub use package_manager::{PackageSpec, PackageInfo, Ecosystem};
pub use config::{ToolConfig, GlobalConfig};
pub use config_figment::{FigmentConfigManager, VxConfig, DefaultConfig, ProjectInfo, ProjectType, ConfigStatus};
pub use error::{VxError, Result};
// pub use registry::{PluginRegistry, ToolRegistry}; // Temporarily disabled during migration

// Re-export utility modules
pub use platform::{Platform, OperatingSystem, Architecture};
pub use http::{HttpUtils, get_http_client};
pub use url_builder::{NodeUrlBuilder, GoUrlBuilder, RustUrlBuilder, PythonUrlBuilder, GenericUrlBuilder};
pub use version_parser::{NodeVersionParser, GoVersionParser, GitHubVersionParser, VersionParserUtils};
pub use version_manager::{Version, VersionManager};
pub use venv::{VenvManager, VenvConfig};


