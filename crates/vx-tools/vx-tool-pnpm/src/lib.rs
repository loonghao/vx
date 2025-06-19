//! PNPM package manager support for vx
//!
//! This provides PNPM package manager integration for the vx tool.

mod config;

pub use config::{
    create_install_config, get_install_methods, get_manual_instructions, supports_auto_install,
    Config, PnpmUrlBuilder,
};

use anyhow::Result;
use std::path::Path;
use vx_plugin::{Ecosystem, PackageSpec, VxPackageManager, VxPlugin, VxTool};

/// PNPM package manager implementation
#[derive(Default)]
pub struct PnpmPackageManager;

impl PnpmPackageManager {
    /// Check if project supports workspaces
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
        vec![]
    }

    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(PnpmPackageManager)]
    }

    fn supports_tool(&self, _tool_name: &str) -> bool {
        false
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
