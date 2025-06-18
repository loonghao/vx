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
//! use vx_core::VxTool;
//! use vx_plugin::{VersionInfo, ToolContext};
//!
//! #[derive(Default)]
//! struct MyTool;
//!
//! #[async_trait::async_trait]
//! impl VxTool for MyTool {
//!     fn name(&self) -> &str { "mytool" }
//!
//!     async fn fetch_versions(&self, _include_prerelease: bool) -> anyhow::Result<Vec<VersionInfo>> {
//!         // Fetch versions from your tool's API
//!         Ok(vec![])
//!     }
//!
//!     async fn install_version(&self, version: &str, force: bool) -> anyhow::Result<()> {
//!         // Download and install the tool
//!         Ok(())
//!     }
//! }
//! ```

pub mod config;
// Tests moved to tests/ directory
pub mod error;
pub mod global_tool_manager;
pub mod install_configs;
pub mod installer_adapter;
pub mod package_manager;
pub mod plugin;
pub mod proxy;
pub mod registry;
pub mod shimexe_integration;
pub mod symlink_venv;
pub mod tool;
pub mod tool_utils;
pub mod version;

// Utility modules
pub mod environment;
pub mod http;
pub mod platform;
pub mod url_builder;
pub mod venv;
// Version management moved to vx-version crate
// pub mod version_manager;
// pub mod version_parser;

// Re-export main traits for convenience
// Plugin system (from vx-plugin crate)
pub use vx_plugin::{
    ConfigurableTool, PluginRegistry as VxPluginRegistry, PluginRegistryBuilder,
    StandardPackageManager, StandardPlugin, ToolMetadata, ToolRegistry as VxToolRegistry,
    UrlBuilder, VersionParser, VxPackageManager, VxPlugin, VxTool,
};
// New plugin system (Plugin-based) - for future migration
pub use config::{GlobalConfig, ToolConfig};
// Re-export configuration types from vx-config
pub use error::{Result, VxError};
pub use install_configs::{
    get_install_config, get_manual_install_instructions, supports_auto_install,
};
pub use installer_adapter::{InstallConfig, InstallProgress, InstallStage};
// Re-export package manager types from vx-plugin (preferred) and vx-core (legacy)
pub use package_manager::{LinuxDistro, SystemType}; // Keep vx-core specific types
                                                    // Re-export registries (vx-plugin versions are preferred)
pub use registry::{PluginRegistry as CorePluginRegistry, ToolRegistry as CoreToolRegistry}; // Legacy vx-core registries
                                                                                            // Note: VxPluginRegistry and VxToolRegistry from vx-plugin are already re-exported above
pub use tool::{
    AsyncTool, Configuration, Environment, Plugin, Tool, ToolContext, ToolExecutionResult,
    ToolInfo, ToolStatus,
};
pub use version::VersionInfo;
// Re-export version management from vx-version crate
pub use vx_config::{
    ConfigManager, ConfigStatus, DefaultConfig, ProjectInfo, ProjectType, VxConfig,
};
pub use vx_plugin::{Ecosystem, IsolationLevel, PackageInfo, PackageManagerConfig, PackageSpec};
pub use vx_version::{
    manager::Version as VxVersion, GitHubVersionFetcher, NodeVersionFetcher, VersionFetcher,
    VersionManager as VxVersionManager, VersionParser as VxVersionParser,
    VersionUtils as VxVersionUtils,
}; // Re-enabled for current system

// Re-export utility modules
pub use environment::{EnvironmentConfig, ToolInstallation, VxEnvironment};
pub use global_tool_manager::{GlobalToolInfo, GlobalToolManager, VenvDependency};
pub use http::{get_http_client, HttpUtils};
pub use installer_adapter::{InstallerAdapter, ToolDownloader};
pub use platform::{Architecture, OperatingSystem, Platform};
pub use proxy::{ProxyContext, ToolProxy};
pub use shimexe_integration::VxShimexeManager;
pub use symlink_venv::{SymlinkVenv, SymlinkVenvManager};
pub use tool_utils::{MetadataUtils, PlatformUtils, UrlUtils, ValidationUtils};
pub use url_builder::{
    GenericUrlBuilder, GoUrlBuilder, NodeUrlBuilder, PythonUrlBuilder, RustUrlBuilder, UvUrlBuilder,
};
pub use venv::{ProjectConfig, ProjectSettings, VenvConfig, VenvManager};
// Version management moved to vx-version crate
// pub use version_manager::{Version, VersionManager};
// pub use version_parser::{
//     GitHubVersionParser, GoVersionParser, NodeVersionParser, VersionParserUtils,
// };
