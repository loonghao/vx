//! Bun package manager and tool support for vx
//!
//! This provides Bun package manager integration and tool support for the vx tool.

pub mod config;
pub mod tool;

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use vx_plugin::{
    Ecosystem, PackageSpec, ToolContext, ToolExecutionResult, VersionInfo, VxPackageManager,
    VxPlugin, VxTool,
};
use vx_version::{TurboCdnVersionFetcher, VersionFetcher};

/// Bun package manager implementation
#[derive(Default)]
pub struct BunPackageManager;

#[async_trait::async_trait]
impl VxPackageManager for BunPackageManager {
    fn name(&self) -> &str {
        "bun"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }

    fn description(&self) -> &str {
        "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
    }

    /// Detect if this is a bun project by looking for bun.lockb
    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        project_path.join("bun.lockb").exists()
    }

    fn get_config_files(&self) -> Vec<&str> {
        vec!["package.json", "bun.lockb", "bunfig.toml"]
    }

    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
        if packages.is_empty() {
            self.run_command(&["install"], &[], project_path).await
        } else {
            let package_names: Vec<String> = packages
                .iter()
                .map(|pkg| {
                    if let Some(version) = &pkg.version {
                        format!("{}@{}", pkg.name, version)
                    } else {
                        pkg.name.clone()
                    }
                })
                .collect();

            let command = if packages.iter().any(|pkg| pkg.dev_dependency) {
                vec!["add", "--dev"]
            } else {
                vec!["add"]
            };

            self.run_command(&command, &package_names, project_path)
                .await
        }
    }

    async fn remove_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        self.run_command(&["remove"], packages, project_path).await
    }

    async fn update_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        if packages.is_empty() {
            self.run_command(&["update"], &[], project_path).await
        } else {
            self.run_command(&["update"], packages, project_path).await
        }
    }
}

/// Bun tool implementation
#[derive(Debug, Clone)]
pub struct BunTool {
    #[allow(dead_code)]
    version_fetcher: Option<TurboCdnVersionFetcher>,
}

impl BunTool {
    pub fn new() -> Self {
        Self {
            version_fetcher: None,
        }
    }

    /// Initialize the tool with turbo-cdn support
    pub async fn init() -> Result<Self> {
        let version_fetcher = TurboCdnVersionFetcher::new("oven-sh", "bun").await?;
        Ok(Self {
            version_fetcher: Some(version_fetcher),
        })
    }

    /// Get or initialize the version fetcher
    #[allow(dead_code)]
    async fn get_version_fetcher(&self) -> Result<TurboCdnVersionFetcher> {
        match &self.version_fetcher {
            Some(fetcher) => Ok(fetcher.clone()),
            None => {
                // Create a new fetcher if not initialized
                TurboCdnVersionFetcher::new("oven-sh", "bun")
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create TurboCdnVersionFetcher: {}", e))
            }
        }
    }

    /// Get version install directory
    fn get_version_install_dir(&self, version: &str) -> std::path::PathBuf {
        let paths = vx_paths::VxPaths::default();
        paths.tools_dir.join("bun").join(version)
    }

    /// Get executable path for a given install directory
    async fn get_executable_path(&self, install_dir: &Path) -> Result<std::path::PathBuf> {
        let exe_name = if cfg!(windows) { "bun.exe" } else { "bun" };
        let exe_path = install_dir.join(exe_name);

        if exe_path.exists() {
            Ok(exe_path)
        } else {
            Err(anyhow::anyhow!(
                "Bun executable not found at {}",
                exe_path.display()
            ))
        }
    }

    /// Ensure tool is available and return executable path
    async fn ensure_available(&self, _context: &ToolContext) -> Result<String> {
        // Try to get active version
        match self.get_active_version().await {
            Ok(version) => {
                let install_dir = self.get_version_install_dir(&version);
                match self.get_executable_path(&install_dir).await {
                    Ok(path) => Ok(path.to_string_lossy().to_string()),
                    Err(_) => {
                        // Install latest if not found
                        self.install_version("latest", false).await?;
                        let latest_version = self.get_active_version().await?;
                        let latest_dir = self.get_version_install_dir(&latest_version);
                        let exe_path = self.get_executable_path(&latest_dir).await?;
                        Ok(exe_path.to_string_lossy().to_string())
                    }
                }
            }
            Err(_) => {
                // No versions installed, install latest
                self.install_version("latest", false).await?;
                let latest_version = self.get_active_version().await?;
                let latest_dir = self.get_version_install_dir(&latest_version);
                let exe_path = self.get_executable_path(&latest_dir).await?;
                Ok(exe_path.to_string_lossy().to_string())
            }
        }
    }
}

impl Default for BunTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl VxTool for BunTool {
    fn name(&self) -> &str {
        "bun"
    }

    fn description(&self) -> &str {
        "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // Use GitHub API directly for reliability with custom version parsing
        use vx_version::GitHubVersionFetcher;
        let fetcher = GitHubVersionFetcher::new("oven-sh", "bun");
        let mut versions = fetcher
            .fetch_versions(include_prerelease)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch bun versions: {}", e))?;

        // Clean Bun version tags (bun-v1.2.3 -> 1.2.3)
        for version in &mut versions {
            if version.version.starts_with("bun-v") {
                version.version = version.version[5..].to_string();
            } else if version.version.starts_with("v") {
                version.version = version.version[1..].to_string();
            }
        }

        Ok(versions)
    }

    async fn install_version(&self, version: &str, force: bool) -> Result<()> {
        // Resolve "latest" to actual version first
        let actual_version = if version == "latest" {
            let versions = self.fetch_versions(false).await?;
            if let Some(latest_version) = versions.first() {
                latest_version.version.clone()
            } else {
                return Err(anyhow::anyhow!("No versions found for {}", self.name()));
            }
        } else {
            version.to_string()
        };

        if !force && self.is_version_installed(&actual_version).await? {
            return Err(anyhow::anyhow!(
                "Version {} of bun is already installed",
                actual_version
            ));
        }

        let install_dir = self.get_version_install_dir(&actual_version);

        // Use real installation with vx-installer (with resolved version)
        let config = crate::config::create_install_config(&actual_version, install_dir).await?;
        let installer = vx_installer::Installer::new().await?;

        let _exe_path = installer
            .install(&config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to install Bun {}: {}", actual_version, e))?;

        // Verify installation
        if !self.is_version_installed(&actual_version).await? {
            return Err(anyhow::anyhow!(
                "Installation verification failed for bun version {}",
                actual_version
            ));
        }

        Ok(())
    }

    async fn is_version_installed(&self, version: &str) -> Result<bool> {
        // Check vx-managed installation first
        let install_dir = self.get_version_install_dir(version);
        if !install_dir.exists() {
            return Ok(false);
        }

        // Check if executable exists in the install directory
        match self.get_executable_path(&install_dir).await {
            Ok(exe_path) => Ok(exe_path.exists()),
            Err(_) => Ok(false),
        }
    }

    async fn execute(&self, args: &[String], context: &ToolContext) -> Result<ToolExecutionResult> {
        // Use standard execution logic with vx-managed installation
        let executable = self.ensure_available(context).await?;

        let mut cmd = std::process::Command::new(&executable);
        cmd.args(args);

        if let Some(cwd) = &context.working_directory {
            cmd.current_dir(cwd);
        }

        for (key, value) in &context.environment_variables {
            cmd.env(key, value);
        }

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute bun: {}", e))?;

        Ok(ToolExecutionResult {
            exit_code: status.code().unwrap_or(1),
            stdout: None,
            stderr: None,
        })
    }

    async fn get_active_version(&self) -> Result<String> {
        // Get the latest installed version
        let installed_versions = self.get_installed_versions().await?;
        if let Some(latest_version) = installed_versions.first() {
            Ok(latest_version.clone())
        } else {
            Err(anyhow::anyhow!("No Bun versions installed"))
        }
    }

    async fn get_installed_versions(&self) -> Result<Vec<String>> {
        let path_manager = vx_paths::PathManager::new().unwrap_or_default();
        let mut versions = path_manager.list_tool_versions("bun")?;

        // Check for system bun asynchronously
        if versions.is_empty() {
            let system_check = tokio::task::spawn_blocking(|| which::which("bun").is_ok());
            if system_check.await.unwrap_or(false) {
                versions.push("system".to_string());
            }
        }

        Ok(versions)
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        // Handle "latest" version by resolving to actual version first
        let actual_version = if version == "latest" {
            let versions = self.fetch_versions(false).await?;
            if let Some(latest_version) = versions.first() {
                latest_version.version.clone()
            } else {
                return Ok(None);
            }
        } else {
            version.to_string()
        };

        // Use global config system for sync access
        use vx_config::get_tool_download_url_sync;

        // Try to get URL from config first
        if let Some(url) = get_tool_download_url_sync("bun", &actual_version) {
            return Ok(Some(url));
        }

        // Fallback to hardcoded logic with actual version
        let platform = if cfg!(windows) {
            "bun-windows-x64"
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "bun-darwin-aarch64"
            } else {
                "bun-darwin-x64"
            }
        } else if cfg!(target_arch = "aarch64") {
            "bun-linux-aarch64"
        } else {
            "bun-linux-x64"
        };

        let url = format!(
            "https://github.com/oven-sh/bun/releases/download/bun-v{}/{}.zip",
            actual_version, platform
        );

        Ok(Some(url))
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://bun.sh/".to_string());
        meta.insert("ecosystem".to_string(), "javascript".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/oven-sh/bun".to_string(),
        );
        meta
    }

    fn get_dependencies(&self) -> Vec<vx_plugin::ToolDependency> {
        // Bun is a standalone JavaScript runtime that doesn't require Node.js
        // It's actually an alternative to Node.js
        vec![]
    }
}

/// Bun plugin
#[derive(Default)]
pub struct BunPlugin;

impl BunPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl VxPlugin for BunPlugin {
    fn name(&self) -> &str {
        "bun"
    }

    fn description(&self) -> &str {
        "Bun package manager support for vx"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![Box::new(BunTool::new())]
    }

    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(BunPackageManager)]
    }

    fn supports_tool(&self, tool_name: &str) -> bool {
        tool_name == "bun"
    }
}

/// Factory function to create the plugin
pub fn create_bun_plugin() -> Box<dyn VxPlugin> {
    Box::new(BunPlugin)
}

// Re-export the new config-based tool
pub use tool::{create_bun_tool, BunConfigTool};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bun_package_manager() {
        let pm = BunPackageManager;
        assert_eq!(pm.name(), "bun");
        assert_eq!(pm.ecosystem(), Ecosystem::Node);
        assert_eq!(
            pm.description(),
            "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
        );
    }

    #[test]
    fn test_bun_tool() {
        let tool = BunTool::new();
        assert_eq!(tool.name(), "bun");
        assert_eq!(
            tool.description(),
            "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
        );
        assert!(tool.aliases().is_empty());

        let metadata = tool.metadata();
        assert_eq!(
            metadata.get("homepage"),
            Some(&"https://bun.sh/".to_string())
        );
        assert_eq!(metadata.get("ecosystem"), Some(&"javascript".to_string()));
        assert_eq!(
            metadata.get("repository"),
            Some(&"https://github.com/oven-sh/bun".to_string())
        );
    }

    #[test]
    fn test_bun_plugin() {
        let plugin = BunPlugin;
        assert_eq!(plugin.name(), "bun");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(plugin.supports_tool("bun"));
        assert!(!plugin.supports_tool("node"));

        let tools = plugin.tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name(), "bun");

        let package_managers = plugin.package_managers();
        assert_eq!(package_managers.len(), 1);
        assert_eq!(package_managers[0].name(), "bun");
    }
}
