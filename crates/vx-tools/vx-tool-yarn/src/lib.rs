//! Yarn package manager support for vx
//!
//! This provides Yarn package manager integration for the vx tool.

use anyhow::Result;
use std::path::Path;
use vx_core::{Ecosystem, PackageSpec, VxPackageManager, VxPlugin, VxTool};

/// Yarn package manager implementation
#[derive(Default)]
pub struct YarnPackageManager;

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
    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        project_path.join("yarn.lock").exists()
    }

    fn get_config_files(&self) -> Vec<&str> {
        vec!["package.json", "yarn.lock", ".yarnrc.yml"]
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
}

/// Yarn plugin
#[derive(Default)]
pub struct YarnPlugin;

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

    #[test]
    fn test_yarn_package_manager() {
        let pm = YarnPackageManager;
        assert_eq!(pm.name(), "yarn");
        assert_eq!(pm.ecosystem(), Ecosystem::JavaScript);
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
}
