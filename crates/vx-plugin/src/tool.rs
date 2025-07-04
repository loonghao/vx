//! Tool plugin trait and related functionality
//!
//! This module defines the `VxTool` trait, which is the core interface for implementing
//! tool support in the vx ecosystem. Tools can be anything from compilers and interpreters
//! to CLI utilities and development tools.

use crate::{Result, ToolContext, ToolExecutionResult, ToolStatus, VersionInfo};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vx_paths::{with_executable_extension, PathManager};

/// Simplified trait for implementing tool support
///
/// This trait provides sensible defaults for most methods, so developers only need
/// to implement the essential functionality for their specific tool.
///
/// # Required Methods
///
/// - `name()`: Return the tool name
/// - `fetch_versions()`: Fetch available versions from the tool's source
///
/// # Optional Methods
///
/// All other methods have default implementations that work for most tools,
/// but can be overridden for custom behavior.
///
/// # Example
///
/// ```rust,no_run
/// use vx_plugin::{VxTool, VersionInfo, Result};
/// use async_trait::async_trait;
///
/// struct MyTool;
///
/// #[async_trait]
/// impl VxTool for MyTool {
///     fn name(&self) -> &str {
///         "mytool"
///     }
///
///     async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
///         // Fetch versions from your tool's API or registry
///         Ok(vec![
///             VersionInfo::new("1.0.0"),
///             VersionInfo::new("1.1.0"),
///         ])
///     }
/// }
/// ```
#[async_trait]
pub trait VxTool: Send + Sync {
    /// Tool name (required)
    ///
    /// This should be a unique identifier for the tool, typically matching
    /// the executable name or common name used to invoke the tool.
    fn name(&self) -> &str;

    /// Tool description (optional, has default)
    ///
    /// A human-readable description of what this tool does.
    fn description(&self) -> &str {
        "A development tool"
    }

    /// Supported aliases for this tool (optional)
    ///
    /// Alternative names that can be used to refer to this tool.
    /// For example, "node" might have aliases like "nodejs".
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    /// Fetch available versions from the tool's official source
    ///
    /// This is the main method developers need to implement. It should
    /// fetch version information from the tool's official source (GitHub releases,
    /// package registry, etc.) and return a list of available versions.
    ///
    /// # Arguments
    ///
    /// * `include_prerelease` - Whether to include prerelease/beta versions
    ///
    /// # Returns
    ///
    /// A vector of `VersionInfo` objects containing version details.
    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>>;

    /// Install a specific version of the tool
    ///
    /// Default implementation provides a basic download-and-extract workflow
    /// that works for most tools. Override this method if your tool requires
    /// special installation procedures.
    ///
    /// # Arguments
    ///
    /// * `version` - The version to install
    /// * `force` - Whether to force reinstallation if already installed
    async fn install_version(&self, version: &str, force: bool) -> Result<()> {
        if !force && self.is_version_installed(version).await? {
            return Err(anyhow::anyhow!(
                "Version {} of {} is already installed. Use --force to reinstall.",
                version,
                self.name()
            ));
        }

        let install_dir = self.get_version_install_dir(version);
        let _exe_path = self.default_install_workflow(version, &install_dir).await?;

        // Verify installation
        if !self.is_version_installed(version).await? {
            return Err(anyhow::anyhow!(
                "Installation verification failed for {} version {}",
                self.name(),
                version
            ));
        }

        Ok(())
    }
    /// Check if a version is installed
    ///
    /// Default implementation checks for the existence of the tool's executable
    /// in the standard vx path structure.
    async fn is_version_installed(&self, version: &str) -> Result<bool> {
        let path_manager = PathManager::new().unwrap_or_else(|_| PathManager::default());
        Ok(path_manager.is_tool_version_installed(self.name(), version))
    }

    /// Execute the tool with given arguments
    ///
    /// Default implementation finds the tool executable and runs it
    /// with the provided arguments and context.
    async fn execute(&self, args: &[String], context: &ToolContext) -> Result<ToolExecutionResult> {
        // Default implementation would use the tool execution logic
        let _ = (args, context);
        Ok(ToolExecutionResult::success())
    }

    /// Get the executable path within an installation directory
    ///
    /// Override this if your tool has a non-standard layout.
    /// The default implementation uses the standard vx path structure.
    async fn get_executable_path(&self, install_dir: &Path) -> Result<PathBuf> {
        let exe_name = with_executable_extension(self.name());

        // For standard vx installations, the executable should be directly in the version directory
        let standard_path = install_dir.join(&exe_name);
        if standard_path.exists() {
            return Ok(standard_path);
        }

        // Try common locations for legacy or non-standard installations
        let candidates = vec![
            install_dir.join("bin").join(&exe_name),
            install_dir.join("Scripts").join(&exe_name), // Windows Python-style
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // Default to standard vx path structure
        Ok(standard_path)
    }

    /// Get download URL for a specific version and current platform
    ///
    /// Override this to provide platform-specific URLs.
    /// The default implementation tries to extract URLs from version info.
    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        let versions = self.fetch_versions(true).await?;
        Ok(versions
            .iter()
            .find(|v| v.version == version)
            .and_then(|v| v.download_url.clone()))
    }
    /// Get installation directory for a specific version
    ///
    /// Returns the path where this version of the tool should be installed.
    /// Uses the standard vx path structure: ~/.vx/tools/<tool>/<version>
    fn get_version_install_dir(&self, version: &str) -> PathBuf {
        // Use PathManager for consistent path structure
        let path_manager = PathManager::new().unwrap_or_else(|_| PathManager::default());
        path_manager.tool_version_dir(self.name(), version)
    }

    /// Get base installation directory for this tool
    ///
    /// Returns the base directory where all versions of this tool are installed.
    /// Uses the standard vx path structure: ~/.vx/tools/<tool>
    fn get_base_install_dir(&self) -> PathBuf {
        // Use PathManager for consistent path structure
        let path_manager = PathManager::new().unwrap_or_else(|_| PathManager::default());
        path_manager.tool_dir(self.name())
    }

    /// Get the currently active version
    ///
    /// Default implementation returns the latest installed version.
    async fn get_active_version(&self) -> Result<String> {
        let installed_versions = self.get_installed_versions().await?;
        installed_versions
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No versions installed for {}", self.name()))
    }

    /// Get all installed versions
    ///
    /// Default implementation uses PathManager to scan for installed versions.
    async fn get_installed_versions(&self) -> Result<Vec<String>> {
        let path_manager = PathManager::new().unwrap_or_else(|_| PathManager::default());
        let mut versions = path_manager.list_tool_versions(self.name())?;

        // Sort versions (newest first)
        versions.sort_by(|a, b| b.cmp(a));
        Ok(versions)
    }
    /// Remove a specific version of the tool
    ///
    /// Default implementation uses PathManager to remove the version.
    async fn remove_version(&self, version: &str, force: bool) -> Result<()> {
        let path_manager = PathManager::new().unwrap_or_else(|_| PathManager::default());

        if !path_manager.is_tool_version_installed(self.name(), version) {
            if !force {
                return Err(anyhow::anyhow!(
                    "Version {} of {} is not installed",
                    version,
                    self.name()
                ));
            }
            return Ok(());
        }

        path_manager.remove_tool_version(self.name(), version)?;
        Ok(())
    }

    /// Get tool status (installed versions, active version, etc.)
    ///
    /// Default implementation gathers status information from other methods.
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

    /// Default installation workflow (download + extract)
    ///
    /// Most tools can use this as-is. This method handles the common pattern
    /// of downloading a tool from a URL and extracting it to the installation directory.
    async fn default_install_workflow(&self, version: &str, install_dir: &Path) -> Result<PathBuf> {
        // Get download URL
        let _download_url = self.get_download_url(version).await?.ok_or_else(|| {
            anyhow::anyhow!(
                "No download URL found for {} version {}",
                self.name(),
                version
            )
        })?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)?;

        // For now, this is a placeholder implementation
        // In a real implementation, this would:
        // 1. Download the file from download_url
        // 2. Extract it to install_dir
        // 3. Set up any necessary symlinks or scripts
        // 4. Return the path to the main executable

        // Create executable in standard vx path structure
        let path_manager = PathManager::new().unwrap_or_else(|_| PathManager::default());
        let exe_path = path_manager.tool_executable_path(self.name(), version);

        if let Some(parent) = exe_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create a placeholder file to indicate installation
        std::fs::write(
            &exe_path,
            format!(
                "#!/bin/bash\necho 'This is {} version {}'\n",
                self.name(),
                version
            ),
        )?;

        // Make it executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&exe_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&exe_path, perms)?;
        }

        Ok(exe_path)
    }

    /// Additional metadata for the tool (optional)
    ///
    /// Override this to provide tool-specific metadata such as
    /// supported platforms, configuration options, etc.
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Helper trait for URL builders that can generate download URLs
pub trait UrlBuilder: Send + Sync {
    /// Generate download URL for a specific version
    fn download_url(&self, version: &str) -> Option<String>;

    /// Get the base URL for fetching version information
    fn versions_url(&self) -> &str;
}

/// Helper trait for version parsers that can parse API responses
pub trait VersionParser: Send + Sync {
    /// Parse version information from JSON response
    fn parse_versions(
        &self,
        json: &serde_json::Value,
        include_prerelease: bool,
    ) -> Result<Vec<VersionInfo>>;
}

/// Configuration-driven tool implementation
///
/// This tool uses configuration to determine download URLs and version sources,
/// making it highly configurable without code changes.
pub struct ConfigurableTool {
    metadata: crate::ToolMetadata,
    url_builder: Box<dyn UrlBuilder>,
    #[allow(dead_code)]
    version_parser: Box<dyn VersionParser>,
}

impl ConfigurableTool {
    /// Create a new configurable tool
    pub fn new(
        metadata: crate::ToolMetadata,
        url_builder: Box<dyn UrlBuilder>,
        version_parser: Box<dyn VersionParser>,
    ) -> Self {
        Self {
            metadata,
            url_builder,
            version_parser,
        }
    }

    /// Get the tool metadata
    pub fn metadata(&self) -> &crate::ToolMetadata {
        &self.metadata
    }
}

#[async_trait]
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
        // For now, we'll use a placeholder implementation
        // In a real implementation, this would fetch from the URL builder's versions URL
        let _ = include_prerelease;

        // Placeholder: return some example versions
        Ok(vec![
            VersionInfo::new("1.0.0"),
            VersionInfo::new("1.1.0"),
            VersionInfo::new("2.0.0"),
        ])
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        // Use the URL builder to generate download URL
        Ok(self.url_builder.download_url(version))
    }

    fn metadata(&self) -> HashMap<String, String> {
        self.metadata.metadata.clone()
    }
}
