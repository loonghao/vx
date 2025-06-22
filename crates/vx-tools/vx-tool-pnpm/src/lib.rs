//! PNPM package manager support for vx
//!
//! This provides PNPM package manager integration for the vx tool.

mod config;

pub use config::{
    create_install_config, get_install_methods, get_manual_instructions, supports_auto_install,
    Config, PnpmUrlBuilder,
};

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use vx_plugin::{
    Ecosystem, PackageSpec, ToolContext, ToolExecutionResult, VersionInfo, VxPackageManager,
    VxPlugin, VxTool,
};
use vx_version::{TurboCdnVersionFetcher, VersionFetcher};

/// PNPM package manager implementation
#[derive(Default)]
pub struct PnpmPackageManager;

impl PnpmPackageManager {
    /// Check if project supports workspaces
    #[allow(dead_code)]
    fn supports_workspaces(&self, project_path: &Path) -> bool {
        // Check for pnpm-workspace.yaml
        if project_path.join("pnpm-workspace.yaml").exists() {
            return true;
        }

        // Check for workspaces in package.json
        if let Ok(package_json) = std::fs::read_to_string(project_path.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&package_json) {
                return json.get("workspaces").is_some();
            }
        }
        false
    }
}

#[async_trait::async_trait]
impl VxPackageManager for PnpmPackageManager {
    fn name(&self) -> &str {
        "pnpm"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }

    fn description(&self) -> &str {
        "Fast, disk space efficient package manager"
    }

    /// Detect if this is a pnpm project by looking for pnpm-lock.yaml
    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        project_path.join("pnpm-lock.yaml").exists()
    }

    fn get_config_files(&self) -> Vec<&str> {
        vec!["package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml"]
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
                vec!["add", "--save-dev"]
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

    async fn list_packages(&self, project_path: &Path) -> Result<Vec<vx_plugin::PackageInfo>> {
        // Use default implementation which attempts to parse common files
        self.default_list_packages(project_path).await
    }
}

/// PNPM tool implementation
#[derive(Debug, Clone)]
pub struct PnpmTool {
    version_fetcher: Option<TurboCdnVersionFetcher>,
}

impl PnpmTool {
    pub fn new() -> Self {
        Self {
            version_fetcher: None,
        }
    }

    /// Initialize the tool with turbo-cdn support
    pub async fn init() -> Result<Self> {
        let version_fetcher = TurboCdnVersionFetcher::new("pnpm", "pnpm").await?;
        Ok(Self {
            version_fetcher: Some(version_fetcher),
        })
    }

    /// Get or initialize the version fetcher
    async fn get_version_fetcher(&self) -> Result<TurboCdnVersionFetcher> {
        match &self.version_fetcher {
            Some(fetcher) => Ok(fetcher.clone()),
            None => {
                // Create a new fetcher if not initialized
                TurboCdnVersionFetcher::new("pnpm", "pnpm")
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create TurboCdnVersionFetcher: {}", e))
            }
        }
    }
}

impl Default for PnpmTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl VxTool for PnpmTool {
    fn name(&self) -> &str {
        "pnpm"
    }

    fn description(&self) -> &str {
        "Fast, disk space efficient package manager"
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // Use TurboCdn for version fetching
        let fetcher = self.get_version_fetcher().await?;
        fetcher
            .fetch_versions(include_prerelease)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch pnpm versions: {}", e))
    }

    async fn install_version(&self, version: &str, force: bool) -> Result<()> {
        if !force && self.is_version_installed(version).await? {
            return Err(anyhow::anyhow!(
                "Version {} of pnpm is already installed",
                version
            ));
        }

        let install_dir = self.get_version_install_dir(version);

        // Use real installation with vx-installer
        let config = crate::config::create_install_config(version, install_dir);
        let installer = vx_installer::Installer::new().await?;

        let _exe_path = installer
            .install(&config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to install PNPM {}: {}", version, e))?;

        // Verify installation
        if !self.is_version_installed(version).await? {
            return Err(anyhow::anyhow!(
                "Installation verification failed for pnpm version {}",
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
        // Determine which pnpm executable to use
        let pnpm_executable = if context.use_system_path {
            "pnpm".to_string() // Use system pnpm
        } else {
            // Ensure pnpm is installed in vx environment
            let active_version = match self.get_active_version().await {
                Ok(version) => version,
                Err(_) => {
                    // No version installed, try to install latest
                    match self.install_version("latest", false).await {
                        Ok(_) => "latest".to_string(),
                        Err(e) => {
                            return Err(anyhow::anyhow!("Failed to install pnpm: {}", e));
                        }
                    }
                }
            };

            // Get the path to the vx-managed pnpm executable
            let install_dir = self.get_version_install_dir(&active_version);
            match self.get_executable_path(&install_dir).await {
                Ok(path) => path.to_string_lossy().to_string(),
                Err(_) => {
                    // Executable not found, try to install
                    match self.install_version("latest", false).await {
                        Ok(_) => {
                            let latest_dir = self.get_version_install_dir("latest");
                            match self.get_executable_path(&latest_dir).await {
                                Ok(path) => path.to_string_lossy().to_string(),
                                Err(e) => {
                                    return Err(anyhow::anyhow!(
                                        "Failed to find pnpm executable after installation: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Failed to install pnpm: {}", e));
                        }
                    }
                }
            }
        };

        let mut cmd = std::process::Command::new(&pnpm_executable);
        cmd.args(args);

        if let Some(cwd) = &context.working_directory {
            cmd.current_dir(cwd);
        }

        for (key, value) in &context.environment_variables {
            cmd.env(key, value);
        }

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute pnpm: {}", e))?;

        Ok(ToolExecutionResult {
            exit_code: status.code().unwrap_or(1),
            stdout: None,
            stderr: None,
        })
    }

    // Use default implementations from VxTool trait
    // async fn get_active_version(&self) -> Result<String>
    // async fn get_installed_versions(&self) -> Result<Vec<String>>

    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        // PNPM releases are available on GitHub
        let url = format!(
            "https://github.com/pnpm/pnpm/releases/download/v{}/pnpm-linux-x64",
            version
        );
        Ok(Some(url))
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://pnpm.io/".to_string());
        meta.insert("ecosystem".to_string(), "javascript".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/pnpm/pnpm".to_string(),
        );
        meta
    }
}

/// PNPM plugin
#[derive(Default)]
pub struct PnpmPlugin;

impl PnpmPlugin {
    /// Create a new PnpmPlugin instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl VxPlugin for PnpmPlugin {
    fn name(&self) -> &str {
        "pnpm"
    }

    fn description(&self) -> &str {
        "PNPM package manager support for vx"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![Box::new(PnpmTool::new())]
    }

    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(PnpmPackageManager)]
    }

    fn supports_tool(&self, tool_name: &str) -> bool {
        tool_name == "pnpm"
    }
}

/// Factory function to create the plugin
pub fn create_pnpm_plugin() -> Box<dyn VxPlugin> {
    Box::new(PnpmPlugin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_pnpm_package_manager() {
        let pm = PnpmPackageManager;
        assert_eq!(pm.name(), "pnpm");
        assert_eq!(pm.ecosystem(), Ecosystem::Node);
        assert_eq!(
            pm.description(),
            "Fast, disk space efficient package manager"
        );
    }

    #[test]
    fn test_pnpm_plugin() {
        let plugin = PnpmPlugin;
        assert_eq!(plugin.name(), "pnpm");
        assert_eq!(plugin.version(), "1.0.0");
    }

    #[test]
    fn test_pnpm_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        let pm = PnpmPackageManager;

        // No pnpm-lock.yaml file
        assert!(!pm.is_preferred_for_project(project_path));

        // Create pnpm-lock.yaml file
        fs::write(project_path.join("pnpm-lock.yaml"), "lockfileVersion: 5.4").unwrap();
        assert!(pm.is_preferred_for_project(project_path));
    }

    #[test]
    fn test_pnpm_workspace_detection() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        let pm = PnpmPackageManager;

        // No workspaces
        fs::write(project_path.join("package.json"), r#"{"name": "test"}"#).unwrap();
        assert!(!pm.supports_workspaces(project_path));

        // With pnpm-workspace.yaml
        fs::write(
            project_path.join("pnpm-workspace.yaml"),
            "packages:\n  - 'packages/*'",
        )
        .unwrap();
        assert!(pm.supports_workspaces(project_path));

        // Remove pnpm-workspace.yaml and test package.json workspaces
        fs::remove_file(project_path.join("pnpm-workspace.yaml")).unwrap();
        fs::write(
            project_path.join("package.json"),
            r#"{"name": "test", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();
        assert!(pm.supports_workspaces(project_path));
    }
}
