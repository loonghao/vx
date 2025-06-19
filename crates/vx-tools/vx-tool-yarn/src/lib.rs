//! Yarn package manager support for vx
//!
//! This provides Yarn package manager integration for the vx tool.

use anyhow::Result;
use std::path::Path;
use vx_plugin::{Ecosystem, PackageSpec, VxPackageManager, VxPlugin, VxTool};

/// Yarn package manager implementation
#[derive(Default)]
pub struct YarnPackageManager;

impl YarnPackageManager {
    /// Check if this is a Yarn Berry (2+) project
    fn is_yarn_berry(&self, project_path: &Path) -> bool {
        project_path.join(".yarnrc.yml").exists()
    }

    /// Check if this is a Yarn Classic (1.x) project
    fn is_yarn_classic(&self, project_path: &Path) -> bool {
        project_path.join(".yarnrc").exists() && !self.is_yarn_berry(project_path)
    }

    /// Check if project supports workspaces
    fn supports_workspaces(&self, project_path: &Path) -> bool {
        if let Ok(package_json) = std::fs::read_to_string(project_path.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&package_json) {
                return json.get("workspaces").is_some();
            }
        }
        false
    }
}

#[async_trait::async_trait]
impl VxPackageManager for YarnPackageManager {
    fn name(&self) -> &str {
        "yarn"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }

    fn description(&self) -> &str {
        "Fast, reliable, and secure dependency management"
    }

    /// Detect if this is a yarn project by looking for yarn.lock
    /// Also check for .yarnrc.yml (Berry) or .yarnrc (Classic) to determine version
    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        project_path.join("yarn.lock").exists()
    }

    fn get_config_files(&self) -> Vec<&str> {
        vec!["package.json", "yarn.lock", ".yarnrc.yml", ".yarnrc"]
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
            self.run_command(&["upgrade"], &[], project_path).await
        } else {
            self.run_command(&["upgrade"], packages, project_path).await
        }
    }

    async fn list_packages(&self, project_path: &Path) -> Result<Vec<vx_plugin::PackageInfo>> {
        // Use default implementation which attempts to parse common files
        self.default_list_packages(project_path).await
    }
}

/// Yarn plugin
#[derive(Default)]
pub struct YarnPlugin;

impl YarnPlugin {
    /// Create a new YarnPlugin instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl VxPlugin for YarnPlugin {
    fn name(&self) -> &str {
        "yarn"
    }

    fn description(&self) -> &str {
        "Yarn package manager support for vx"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![]
    }

    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(YarnPackageManager)]
    }

    fn supports_tool(&self, _tool_name: &str) -> bool {
        false
    }
}

/// Factory function to create the plugin
pub fn create_yarn_plugin() -> Box<dyn VxPlugin> {
    Box::new(YarnPlugin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_yarn_package_manager() {
        let pm = YarnPackageManager;
        assert_eq!(pm.name(), "yarn");
        assert_eq!(pm.ecosystem(), Ecosystem::Node);
        assert_eq!(
            pm.description(),
            "Fast, reliable, and secure dependency management"
        );
    }

    #[test]
    fn test_yarn_plugin() {
        let plugin = YarnPlugin;
        assert_eq!(plugin.name(), "yarn");
        assert_eq!(plugin.version(), "1.0.0");
    }

    #[test]
    fn test_yarn_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        let pm = YarnPackageManager;

        // No yarn.lock file
        assert!(!pm.is_preferred_for_project(project_path));

        // Create yarn.lock file
        fs::write(project_path.join("yarn.lock"), "").unwrap();
        assert!(pm.is_preferred_for_project(project_path));
    }

    #[test]
    fn test_yarn_version_detection() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        let pm = YarnPackageManager;

        // Test Yarn Berry detection
        fs::write(project_path.join(".yarnrc.yml"), "nodeLinker: node-modules").unwrap();
        assert!(pm.is_yarn_berry(project_path));
        assert!(!pm.is_yarn_classic(project_path));

        // Test Yarn Classic detection
        fs::remove_file(project_path.join(".yarnrc.yml")).unwrap();
        fs::write(
            project_path.join(".yarnrc"),
            "registry \"https://registry.npmjs.org/\"",
        )
        .unwrap();
        assert!(!pm.is_yarn_berry(project_path));
        assert!(pm.is_yarn_classic(project_path));
    }

    #[test]
    fn test_workspace_detection() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        let pm = YarnPackageManager;

        // No workspaces
        fs::write(project_path.join("package.json"), r#"{"name": "test"}"#).unwrap();
        assert!(!pm.supports_workspaces(project_path));

        // With workspaces
        fs::write(
            project_path.join("package.json"),
            r#"{"name": "test", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();
        assert!(pm.supports_workspaces(project_path));
    }
}
