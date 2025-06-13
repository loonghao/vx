//! High-level plugin traits for easy implementation
//!
//! This module provides simplified traits that abstract away most of the complexity,
//! allowing developers to focus on the core functionality of their tools.

use crate::{
    Ecosystem, FigmentConfigManager, HttpUtils, PackageInfo, PackageSpec, Result, ToolContext,
    ToolDownloader, ToolExecutionResult, ToolStatus, VersionInfo, VxEnvironment,
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Simplified trait for implementing tool support
///
/// This trait provides sensible defaults for most methods, so developers only need
/// to implement the essential functionality for their specific tool.
#[async_trait::async_trait]
pub trait VxTool: Send + Sync {
    /// Tool name (required)
    fn name(&self) -> &str;

    /// Tool description (optional, has default)
    fn description(&self) -> &str {
        "A development tool"
    }

    /// Supported aliases for this tool (optional)
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    /// Fetch available versions from the tool's official source
    /// This is the main method developers need to implement
    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>>;

    /// Install a specific version of the tool
    /// Default implementation provides a basic download-and-extract workflow
    async fn install_version(&self, version: &str, force: bool) -> Result<()> {
        if !force && self.is_version_installed(version).await? {
            return Err(crate::VxError::VersionAlreadyInstalled {
                tool_name: self.name().to_string(),
                version: version.to_string(),
            });
        }

        let install_dir = self.get_version_install_dir(version);
        let _exe_path = self.default_install_workflow(version, &install_dir).await?;

        // Verify installation
        if !self.is_version_installed(version).await? {
            return Err(crate::VxError::InstallationFailed {
                tool_name: self.name().to_string(),
                version: version.to_string(),
                message: "Installation verification failed".to_string(),
            });
        }

        Ok(())
    }

    /// Check if a version is installed (has sensible default)
    async fn is_version_installed(&self, version: &str) -> Result<bool> {
        let env = VxEnvironment::new().expect("Failed to create VX environment");
        Ok(env.is_version_installed(self.name(), version))
    }

    /// Execute the tool with given arguments (has default implementation)
    async fn execute(&self, args: &[String], context: &ToolContext) -> Result<ToolExecutionResult> {
        self.default_execute_workflow(args, context).await
    }

    /// Get the executable path within an installation directory
    /// Override this if your tool has a non-standard layout
    async fn get_executable_path(&self, install_dir: &Path) -> Result<PathBuf> {
        let exe_name = if cfg!(target_os = "windows") {
            format!("{}.exe", self.name())
        } else {
            self.name().to_string()
        };

        // Try common locations
        let candidates = vec![
            install_dir.join(&exe_name),
            install_dir.join("bin").join(&exe_name),
            install_dir.join("Scripts").join(&exe_name), // Windows Python-style
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // Default to bin directory
        Ok(install_dir.join("bin").join(exe_name))
    }

    /// Get download URL for a specific version and current platform
    /// Override this to provide platform-specific URLs
    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        // Default: try to extract from version info
        let versions = self.fetch_versions(true).await?;
        Ok(versions
            .iter()
            .find(|v| v.version == version)
            .and_then(|v| v.download_url.clone()))
    }

    /// Get installation directory for a specific version
    fn get_version_install_dir(&self, version: &str) -> PathBuf {
        let env = VxEnvironment::new().expect("Failed to create VX environment");
        env.get_version_install_dir(self.name(), version)
    }

    /// Get base installation directory for this tool
    fn get_base_install_dir(&self) -> PathBuf {
        let env = VxEnvironment::new().expect("Failed to create VX environment");
        env.get_tool_install_dir(self.name())
    }

    /// Default installation workflow (download + extract)
    /// Most tools can use this as-is
    async fn default_install_workflow(
        &self,
        version: &str,
        _install_dir: &Path,
    ) -> Result<PathBuf> {
        // Get download URL
        let download_url = self.get_download_url(version).await?.ok_or_else(|| {
            crate::VxError::DownloadUrlNotFound {
                tool_name: self.name().to_string(),
                version: version.to_string(),
            }
        })?;

        // Use the new downloader
        let downloader = ToolDownloader::new()?;
        downloader
            .download_and_install(self.name(), version, &download_url)
            .await
    }

    /// Default execution workflow
    async fn default_execute_workflow(
        &self,
        args: &[String],
        context: &ToolContext,
    ) -> Result<ToolExecutionResult> {
        // Find the tool executable
        let exe_path = if context.use_system_path {
            which::which(self.name()).map_err(|_| crate::VxError::ToolNotFound {
                tool_name: self.name().to_string(),
            })?
        } else {
            // Use vx-managed version
            let active_version = self.get_active_version().await?;
            let install_dir = self.get_version_install_dir(&active_version);
            let env = VxEnvironment::new().expect("Failed to create VX environment");
            env.find_executable_in_dir(&install_dir, self.name())?
        };

        // Execute the tool
        let mut cmd = std::process::Command::new(&exe_path);
        cmd.args(args);

        if let Some(cwd) = &context.working_directory {
            cmd.current_dir(cwd);
        }

        for (key, value) in &context.environment_variables {
            cmd.env(key, value);
        }

        let status = cmd.status().map_err(|e| crate::VxError::Other {
            message: format!("Failed to execute {}: {}", self.name(), e),
        })?;

        Ok(ToolExecutionResult {
            exit_code: status.code().unwrap_or(1),
            stdout: None, // Could be enhanced to capture output
            stderr: None,
        })
    }

    /// Get the currently active version (has default implementation)
    async fn get_active_version(&self) -> Result<String> {
        let env = VxEnvironment::new().expect("Failed to create VX environment");

        // Try to get from environment config first
        if let Some(active_version) = env.get_active_version(self.name())? {
            return Ok(active_version);
        }

        // Fallback to latest installed
        let installed_versions = self.get_installed_versions().await?;
        installed_versions
            .first()
            .cloned()
            .ok_or_else(|| crate::VxError::ToolNotInstalled {
                tool_name: self.name().to_string(),
            })
    }

    /// Get all installed versions
    async fn get_installed_versions(&self) -> Result<Vec<String>> {
        let env = VxEnvironment::new().expect("Failed to create VX environment");
        env.list_installed_versions(self.name())
    }

    /// Remove a specific version of the tool
    async fn remove_version(&self, version: &str, force: bool) -> Result<()> {
        let version_dir = self.get_version_install_dir(version);

        // Check if the directory exists first
        if !version_dir.exists() {
            if !force {
                return Err(crate::VxError::VersionNotInstalled {
                    tool_name: self.name().to_string(),
                    version: version.to_string(),
                });
            }
            // In force mode, if directory doesn't exist, consider it already removed
            return Ok(());
        }

        // Attempt to remove the directory
        match std::fs::remove_dir_all(&version_dir) {
            Ok(()) => Ok(()),
            Err(e) => {
                if force {
                    // In force mode, ignore certain types of errors
                    match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            // Directory was removed between our check and removal attempt
                            Ok(())
                        }
                        std::io::ErrorKind::PermissionDenied => {
                            // Still report permission errors even in force mode
                            Err(crate::VxError::PermissionError {
                                message: format!(
                                    "Permission denied when removing {} {}: {}",
                                    self.name(),
                                    version,
                                    e
                                ),
                            })
                        }
                        _ => {
                            // For other errors in force mode, convert to a more user-friendly message
                            Err(crate::VxError::IoError {
                                message: format!(
                                    "Failed to remove {} {} directory: {}",
                                    self.name(),
                                    version,
                                    e
                                ),
                            })
                        }
                    }
                } else {
                    // In non-force mode, propagate the original error
                    Err(e.into())
                }
            }
        }
    }

    /// Get tool status (installed versions, active version, etc.)
    async fn get_status(&self) -> Result<ToolStatus> {
        let installed_versions = self.get_installed_versions().await?;
        let current_version = if !installed_versions.is_empty() {
            self.get_active_version().await.ok()
        } else {
            None
        };

        Ok(ToolStatus {
            installed: !installed_versions.is_empty(),
            current_version,
            installed_versions,
        })
    }

    /// Additional metadata for the tool (optional)
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Trait for URL builders that can generate download URLs
pub trait UrlBuilder: Send + Sync {
    fn download_url(&self, version: &str) -> Option<String>;
    fn versions_url(&self) -> &str;
}

/// Trait for version parsers that can parse API responses
pub trait VersionParser: Send + Sync {
    fn parse_versions(&self, json: &Value, include_prerelease: bool) -> Result<Vec<VersionInfo>>;
}

/// Simplified trait for implementing package manager support
///
/// This trait provides a high-level interface for package managers,
/// with sensible defaults for common operations.
#[async_trait::async_trait]
pub trait VxPackageManager: Send + Sync {
    /// Package manager name (required)
    fn name(&self) -> &str;

    /// Ecosystem this package manager belongs to (required)
    fn ecosystem(&self) -> Ecosystem;

    /// Description of the package manager (optional)
    fn description(&self) -> &str {
        "A package manager"
    }

    /// Check if this package manager is available on the system
    async fn is_available(&self) -> Result<bool> {
        // Default: check if the executable exists in PATH
        Ok(which::which(self.name()).is_ok())
    }

    /// Check if this package manager should be used for a project
    /// Override this to detect project-specific files (package.json, Cargo.toml, etc.)
    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        // Default: check for common config files
        let config_files = self.get_config_files();
        config_files
            .iter()
            .any(|file| project_path.join(file).exists())
    }

    /// Get configuration files that indicate this package manager should be used
    fn get_config_files(&self) -> Vec<&str> {
        vec![]
    }

    /// Install packages (main method to implement)
    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()>;

    /// Remove packages (has default implementation)
    async fn remove_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        self.run_command(&["remove"], packages, project_path).await
    }

    /// Update packages (has default implementation)
    async fn update_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        if packages.is_empty() {
            self.run_command(&["update"], &[], project_path).await
        } else {
            self.run_command(&["update"], packages, project_path).await
        }
    }

    /// List installed packages (has default implementation)
    async fn list_packages(&self, project_path: &Path) -> Result<Vec<PackageInfo>> {
        // Default: try to parse from common files or run list command
        self.default_list_packages(project_path).await
    }

    /// Search for packages (has default implementation)
    async fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>> {
        self.run_search_command(query).await
    }

    /// Run a package manager command with arguments
    async fn run_command(
        &self,
        command: &[&str],
        args: &[String],
        project_path: &Path,
    ) -> Result<()> {
        let mut cmd = std::process::Command::new(self.name());
        cmd.args(command);
        cmd.args(args);
        cmd.current_dir(project_path);

        let status = cmd
            .status()
            .map_err(|e| crate::VxError::PackageManagerError {
                manager: self.name().to_string(),
                message: format!("Failed to run command: {}", e),
            })?;

        if !status.success() {
            return Err(crate::VxError::PackageManagerError {
                manager: self.name().to_string(),
                message: format!("Command failed with exit code: {:?}", status.code()),
            });
        }

        Ok(())
    }

    /// Default implementation for listing packages
    async fn default_list_packages(&self, _project_path: &Path) -> Result<Vec<PackageInfo>> {
        // This would be implemented based on common patterns
        // For now, return empty list
        Ok(vec![])
    }

    /// Default implementation for searching packages
    async fn run_search_command(&self, query: &str) -> Result<Vec<PackageInfo>> {
        // This would run the package manager's search command and parse output
        // For now, return empty list
        let _ = query;
        Ok(vec![])
    }

    /// Get the command to install packages (override for custom install commands)
    fn get_install_command(&self) -> Vec<&str> {
        vec!["install"]
    }

    /// Get the command to add new packages (override if different from install)
    fn get_add_command(&self) -> Vec<&str> {
        vec!["add"]
    }

    /// Additional metadata for the package manager (optional)
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Combined plugin trait that can provide both tools and package managers
///
/// This is the main trait that plugin developers implement to register their functionality.
#[async_trait::async_trait]
pub trait VxPlugin: Send + Sync {
    /// Plugin name (required)
    fn name(&self) -> &str;

    /// Plugin description (optional)
    fn description(&self) -> &str {
        "A vx plugin"
    }

    /// Plugin version (optional)
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Get all tools provided by this plugin
    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![]
    }

    /// Get all package managers provided by this plugin
    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![]
    }

    /// Initialize the plugin (optional)
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Check if this plugin supports a specific tool
    fn supports_tool(&self, tool_name: &str) -> bool {
        self.tools()
            .iter()
            .any(|tool| tool.name() == tool_name || tool.aliases().contains(&tool_name))
    }

    /// Check if this plugin supports a specific package manager
    fn supports_package_manager(&self, pm_name: &str) -> bool {
        self.package_managers()
            .iter()
            .any(|pm| pm.name() == pm_name)
    }

    /// Plugin metadata (optional)
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Standard plugin implementation for single-tool plugins
pub struct StandardPlugin {
    name: String,
    description: String,
    version: String,
    tool_factory: Box<dyn Fn() -> Box<dyn VxTool> + Send + Sync>,
}

impl StandardPlugin {
    pub fn new<F>(name: String, description: String, version: String, tool_factory: F) -> Self
    where
        F: Fn() -> Box<dyn VxTool> + Send + Sync + 'static,
    {
        Self {
            name,
            description,
            version,
            tool_factory: Box::new(tool_factory),
        }
    }
}

#[async_trait::async_trait]
impl VxPlugin for StandardPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![(self.tool_factory)()]
    }
}

/// Configuration-driven tool implementation
///
/// This tool uses figment configuration to determine download URLs and version sources,
/// making it highly configurable without code changes.
pub struct ConfigurableTool {
    metadata: ToolMetadata,
    config_manager: FigmentConfigManager,
    url_builder: Box<dyn UrlBuilder>,
    version_parser: Box<dyn VersionParser>,
}

impl ConfigurableTool {
    pub fn new(
        metadata: ToolMetadata,
        url_builder: Box<dyn UrlBuilder>,
        version_parser: Box<dyn VersionParser>,
    ) -> Result<Self> {
        let config_manager =
            FigmentConfigManager::new().or_else(|_| FigmentConfigManager::minimal())?;

        Ok(Self {
            metadata,
            config_manager,
            url_builder,
            version_parser,
        })
    }
}

#[async_trait::async_trait]
impl VxTool for ConfigurableTool {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn description(&self) -> &str {
        &self.metadata.description
    }

    fn aliases(&self) -> Vec<&str> {
        self.metadata.aliases.iter().map(|s| s.as_str()).collect()
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        let json = HttpUtils::fetch_json(self.url_builder.versions_url()).await?;
        self.version_parser
            .parse_versions(&json, include_prerelease)
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        // First try to get from figment configuration
        if let Ok(url) = self
            .config_manager
            .get_download_url(&self.metadata.name, version)
        {
            return Ok(Some(url));
        }

        // Fall back to URL builder
        Ok(self.url_builder.download_url(version))
    }
}

/// Basic tool metadata for standard tools
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
}
