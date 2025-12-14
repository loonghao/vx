//! Tool trait definition
//!
//! The `Tool` trait is the core interface for implementing tool support in vx.

use crate::{Result, ToolContext, ToolDependency, ToolExecutionResult, ToolStatus, VersionInfo};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Core trait for implementing tool support
///
/// This trait provides sensible defaults for most methods, so developers only need
/// to implement the essential functionality for their specific tool.
///
/// # Required Methods
///
/// - `name()`: Return the tool name
/// - `fetch_versions()`: Fetch available versions from the tool's source
///
/// # Example
///
/// ```rust,no_run
/// use vx_sdk::{Tool, VersionInfo, Result};
/// use async_trait::async_trait;
///
/// struct MyTool;
///
/// #[async_trait]
/// impl Tool for MyTool {
///     fn name(&self) -> &str {
///         "mytool"
///     }
///
///     async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
///         Ok(vec![VersionInfo::new("1.0.0")])
///     }
/// }
/// ```
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (required)
    fn name(&self) -> &str;

    /// Tool description (optional)
    fn description(&self) -> &str {
        "A development tool"
    }

    /// Supported aliases for this tool
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    /// Fetch available versions from the tool's official source
    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>>;

    /// Install a specific version of the tool
    async fn install_version(&self, version: &str, force: bool) -> Result<()> {
        if !force && self.is_version_installed(version).await? {
            return Err(anyhow::anyhow!(
                "Version {} of {} is already installed. Use --force to reinstall.",
                version,
                self.name()
            ));
        }

        let install_dir = self.get_version_install_dir(version);
        self.default_install_workflow(version, &install_dir).await?;

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
    async fn is_version_installed(&self, version: &str) -> Result<bool> {
        let install_dir = self.get_version_install_dir(version);
        Ok(install_dir.exists())
    }

    /// Execute the tool with given arguments
    async fn execute(&self, args: &[String], context: &ToolContext) -> Result<ToolExecutionResult> {
        let _ = (args, context);
        Ok(ToolExecutionResult::success())
    }

    /// Get the executable path within an installation directory
    async fn get_executable_path(&self, install_dir: &Path) -> Result<PathBuf> {
        let exe_name = if cfg!(windows) {
            format!("{}.exe", self.name())
        } else {
            self.name().to_string()
        };

        let standard_path = install_dir.join(&exe_name);
        if standard_path.exists() {
            return Ok(standard_path);
        }

        let candidates = vec![
            install_dir.join("bin").join(&exe_name),
            install_dir.join("Scripts").join(&exe_name),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        Ok(standard_path)
    }

    /// Get download URL for a specific version
    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        let versions = self.fetch_versions(true).await?;
        Ok(versions
            .iter()
            .find(|v| v.version == version)
            .and_then(|v| v.download_url.clone()))
    }

    /// Get installation directory for a specific version
    fn get_version_install_dir(&self, version: &str) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".vx")
            .join("tools")
            .join(self.name())
            .join(version)
    }

    /// Get base installation directory for this tool
    fn get_base_install_dir(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".vx").join("tools").join(self.name())
    }

    /// Get the currently active version
    async fn get_active_version(&self) -> Result<String> {
        let installed_versions = self.get_installed_versions().await?;
        installed_versions
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No versions installed for {}", self.name()))
    }

    /// Get all installed versions
    async fn get_installed_versions(&self) -> Result<Vec<String>> {
        let base_dir = self.get_base_install_dir();
        if !base_dir.exists() {
            return Ok(vec![]);
        }

        let mut versions = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&base_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        versions.push(name.to_string());
                    }
                }
            }
        }

        versions.sort_by(|a, b| b.cmp(a));
        Ok(versions)
    }

    /// Remove a specific version of the tool
    async fn remove_version(&self, version: &str, force: bool) -> Result<()> {
        let install_dir = self.get_version_install_dir(version);

        if !install_dir.exists() {
            if !force {
                return Err(anyhow::anyhow!(
                    "Version {} of {} is not installed",
                    version,
                    self.name()
                ));
            }
            return Ok(());
        }

        std::fs::remove_dir_all(&install_dir)?;
        Ok(())
    }

    /// Get tool status
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

    /// Default installation workflow
    async fn default_install_workflow(&self, version: &str, install_dir: &Path) -> Result<PathBuf> {
        let _download_url = self.get_download_url(version).await?.ok_or_else(|| {
            anyhow::anyhow!(
                "No download URL found for {} version {}",
                self.name(),
                version
            )
        })?;

        std::fs::create_dir_all(install_dir)?;

        let exe_name = if cfg!(windows) {
            format!("{}.exe", self.name())
        } else {
            self.name().to_string()
        };
        let exe_path = install_dir.join(&exe_name);

        // Placeholder - real implementation would download and extract
        std::fs::write(
            &exe_path,
            format!(
                "#!/bin/bash\necho 'This is {} version {}'\n",
                self.name(),
                version
            ),
        )?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&exe_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&exe_path, perms)?;
        }

        Ok(exe_path)
    }

    /// Additional metadata for the tool
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Get dependencies for this tool
    fn get_dependencies(&self) -> Vec<ToolDependency> {
        vec![]
    }

    /// Resolve "latest" version to actual version string
    async fn resolve_version(&self, version: &str) -> Result<String> {
        if version == "latest" {
            let versions = self.fetch_versions(false).await?;
            versions
                .first()
                .map(|v| v.version.clone())
                .ok_or_else(|| anyhow::anyhow!("No versions found for {}", self.name()))
        } else {
            Ok(version.to_string())
        }
    }
}

/// Helper trait for URL builders
pub trait UrlBuilder: Send + Sync {
    /// Generate download URL for a specific version
    fn download_url(&self, version: &str) -> Option<String>;

    /// Get the base URL for fetching version information
    fn versions_url(&self) -> &str;
}

/// Helper trait for version parsers
pub trait VersionParser: Send + Sync {
    /// Parse version information from JSON response
    fn parse_versions(
        &self,
        json: &serde_json::Value,
        include_prerelease: bool,
    ) -> Result<Vec<VersionInfo>>;
}
