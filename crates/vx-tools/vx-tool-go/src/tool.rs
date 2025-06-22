//! Go tool implementation

use crate::config::GoUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use vx_plugin::{
    ExecutableTool, InstallableTool, ToolContext, ToolExecutionResult, VersionInfo, VersionedTool,
    VxTool,
};
use vx_tool_standard::StandardUrlBuilder;
use vx_version::{GoVersionFetcher, VersionFetcher};

/// Go tool implementation
#[derive(Debug, Clone)]
pub struct GoTool {
    version_fetcher: GoVersionFetcher,
}

impl GoTool {
    pub fn new() -> Self {
        Self {
            version_fetcher: GoVersionFetcher::new(),
        }
    }
}

impl Default for GoTool {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the new common traits
#[async_trait]
impl InstallableTool for GoTool {
    fn tool_name(&self) -> &str {
        "go"
    }

    async fn is_version_installed(&self, version: &str) -> Result<bool> {
        let install_dir = InstallableTool::get_version_install_dir(self, version);
        if !install_dir.exists() {
            return Ok(false);
        }

        // Check if executable exists in the install directory
        match InstallableTool::get_executable_path(self, &install_dir).await {
            Ok(exe_path) => Ok(exe_path.exists()),
            Err(_) => Ok(false),
        }
    }

    fn get_version_install_dir(&self, version: &str) -> PathBuf {
        let paths = vx_paths::VxPaths::default();
        paths.tools_dir.join("go").join(version)
    }

    async fn get_executable_path(&self, install_dir: &PathBuf) -> Result<PathBuf> {
        let exe_name = if cfg!(windows) { "go.exe" } else { "go" };
        let exe_path = install_dir.join("bin").join(exe_name);

        if exe_path.exists() {
            Ok(exe_path)
        } else {
            Err(anyhow::anyhow!(
                "Go executable not found at {}",
                exe_path.display()
            ))
        }
    }

    async fn create_install_config(
        &self,
        version: &str,
        install_dir: PathBuf,
    ) -> Result<vx_installer::InstallConfig> {
        crate::config::create_install_config(version, install_dir).await
    }

    async fn resolve_version(&self, version: &str) -> Result<String> {
        if version == "latest" {
            if let Some(latest) = self.version_fetcher.get_latest_version().await? {
                Ok(latest.version)
            } else {
                Err(anyhow::anyhow!("No versions found for Go"))
            }
        } else {
            Ok(version.to_string())
        }
    }
}

#[async_trait]
impl ExecutableTool for GoTool {
    fn tool_name(&self) -> &str {
        "go"
    }

    async fn ensure_available(&self, _context: &ToolContext) -> Result<String> {
        // Try to get active version
        match VersionedTool::get_active_version(self).await {
            Ok(version) => {
                let install_dir = InstallableTool::get_version_install_dir(self, &version);
                match InstallableTool::get_executable_path(self, &install_dir).await {
                    Ok(path) => Ok(path.to_string_lossy().to_string()),
                    Err(_) => {
                        // Install latest if not found
                        InstallableTool::install_version_impl(self, "latest", false).await?;
                        let latest_version =
                            InstallableTool::resolve_version(self, "latest").await?;
                        let latest_dir =
                            InstallableTool::get_version_install_dir(self, &latest_version);
                        let exe_path =
                            InstallableTool::get_executable_path(self, &latest_dir).await?;
                        Ok(exe_path.to_string_lossy().to_string())
                    }
                }
            }
            Err(_) => {
                // No version installed, install latest
                InstallableTool::install_version_impl(self, "latest", false).await?;
                let latest_version = InstallableTool::resolve_version(self, "latest").await?;
                let latest_dir = InstallableTool::get_version_install_dir(self, &latest_version);
                let exe_path = InstallableTool::get_executable_path(self, &latest_dir).await?;
                Ok(exe_path.to_string_lossy().to_string())
            }
        }
    }
}

#[async_trait]
impl VersionedTool for GoTool {
    fn tool_name(&self) -> &str {
        "go"
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        self.version_fetcher
            .fetch_versions(include_prerelease)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch Go versions: {}", e))
    }

    async fn get_installed_versions(&self) -> Result<Vec<String>> {
        // Use PathManager to scan for installed versions
        let path_manager = vx_paths::PathManager::new().unwrap_or_default();
        let mut versions = path_manager.list_tool_versions(self.name())?;

        // Sort versions (newest first)
        versions.sort_by(|a, b| b.cmp(a));
        Ok(versions)
    }

    async fn get_active_version(&self) -> Result<String> {
        // Get the latest installed version
        let installed_versions = VersionedTool::get_installed_versions(self).await?;
        if let Some(latest_version) = installed_versions.first() {
            Ok(latest_version.clone())
        } else {
            Err(anyhow::anyhow!("No Go versions installed"))
        }
    }
}

#[async_trait::async_trait]
impl VxTool for GoTool {
    fn name(&self) -> &str {
        "go"
    }

    fn description(&self) -> &str {
        "Go programming language"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["golang"]
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // Delegate to VersionedTool trait
        VersionedTool::fetch_versions(self, include_prerelease).await
    }

    async fn install_version(&self, version: &str, force: bool) -> Result<()> {
        // Delegate to InstallableTool trait
        InstallableTool::install_version_impl(self, version, force).await
    }

    async fn execute(&self, args: &[String], context: &ToolContext) -> Result<ToolExecutionResult> {
        // Delegate to ExecutableTool trait
        ExecutableTool::execute_impl(self, args, context).await
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        Ok(GoUrlBuilder::download_url(version))
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://golang.org/".to_string());
        meta.insert("ecosystem".to_string(), "go".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/golang/go".to_string(),
        );
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_tool_basic() {
        let tool = GoTool::default();

        assert_eq!(tool.name(), "go");
        assert_eq!(tool.description(), "Go programming language");
        assert!(tool.aliases().contains(&"golang"));
    }

    #[test]
    fn test_go_tool_metadata() {
        let tool = GoTool::new();
        let metadata = tool.metadata();

        assert!(metadata.contains_key("homepage"));
        assert!(metadata.contains_key("ecosystem"));
        assert_eq!(metadata.get("ecosystem"), Some(&"go".to_string()));
    }
}
