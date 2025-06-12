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
//!     async fn install_version(&self, version: &str, force: bool) -> Result<()> {
//!         // Download and install the tool
//!         Ok(())
//!     }
//! }
//! ```

pub mod config;
pub mod config_figment;
pub mod error;
pub mod install_configs;
pub mod installer;
pub mod package_manager;
pub mod plugin; // Re-enabled for current system
pub mod registry;
pub mod tool;
pub mod version; // Re-enabled for current system

// Utility modules
pub mod downloader;
pub mod environment;
pub mod http;
pub mod platform;
pub mod url_builder;
pub mod venv;
pub mod version_manager;
pub mod version_parser;

// Re-export main traits for convenience
// Current plugin system (VxPlugin-based)
pub use plugin::{
    ConfigurableTool, StandardPlugin, ToolMetadata, UrlBuilder, VersionParser, VxPackageManager,
    VxPlugin, VxTool,
};
// New plugin system (Plugin-based) - for future migration
pub use config::{GlobalConfig, ToolConfig};
pub use config_figment::{
    ConfigStatus, DefaultConfig, FigmentConfigManager, ProjectInfo, ProjectType, VxConfig,
};
pub use error::{Result, VxError};
pub use install_configs::{
    get_install_config, get_manual_install_instructions, supports_auto_install,
};
pub use installer::{InstallConfig, InstallProgress, InstallStage};
pub use package_manager::{Ecosystem, PackageInfo, PackageSpec};
pub use registry::{PluginRegistry, ToolRegistry};
pub use tool::{
    AsyncTool, Configuration, Environment, Plugin, Tool, ToolContext, ToolExecutionResult,
    ToolInfo, ToolStatus,
};
pub use version::VersionInfo; // Re-enabled for current system

// Re-export utility modules
pub use downloader::ToolDownloader;
pub use environment::{EnvironmentConfig, ToolInstallation, VxEnvironment};
pub use http::{get_http_client, HttpUtils};
pub use platform::{Architecture, OperatingSystem, Platform};
pub use url_builder::{
    GenericUrlBuilder, GoUrlBuilder, NodeUrlBuilder, PythonUrlBuilder, RustUrlBuilder, UvUrlBuilder,
};
pub use venv::{VenvConfig, VenvManager};
pub use version_manager::{Version, VersionManager};
pub use version_parser::{
    GitHubVersionParser, GoVersionParser, NodeVersionParser, VersionParserUtils,
};
