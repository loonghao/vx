//! Bun package manager and tool support for vx
//!
//! This provides Bun package manager integration and tool support for the vx tool.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use vx_plugin::{
    Ecosystem, PackageSpec, ToolContext, ToolExecutionResult, VersionInfo, VxPackageManager,
    VxPlugin, VxTool,
};
use vx_version::{GitHubVersionFetcher, VersionFetcher};

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
    version_fetcher: GitHubVersionFetcher,
}

impl BunTool {
    pub fn new() -> Self {
        Self {
            version_fetcher: GitHubVersionFetcher::new("oven-sh", "bun"),
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
        self.version_fetcher
            .fetch_versions(include_prerelease)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch bun versions: {}", e))
    }

    async fn install_version(&self, version: &str, force: bool) -> Result<()> {
        if !force && self.is_version_installed(version).await? {
            return Err(anyhow::anyhow!(
                "Version {} of bun is already installed",
                version
            ));
        }

        let install_dir = self.get_version_install_dir(version);
        let _exe_path = self.default_install_workflow(version, &install_dir).await?;

        // Verify installation
        if !self.is_version_installed(version).await? {
            return Err(anyhow::anyhow!(
                "Installation verification failed for bun version {}",
                version
            ));
        }

        Ok(())
    }

    async fn is_version_installed(&self, version: &str) -> Result<bool> {
        let install_dir = self.get_version_install_dir(version);
        Ok(install_dir.exists())
    }

    async fn execute(&self, args: &[String], context: &ToolContext) -> Result<ToolExecutionResult> {
        let mut cmd = std::process::Command::new("bun");
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
        Ok("latest".to_string())
    }

    async fn get_installed_versions(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        // Bun releases are available on GitHub
        let url = format!(
            "https://github.com/oven-sh/bun/releases/download/bun-v{}/bun-linux-x64.zip",
            version
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
