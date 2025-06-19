//! Bun package manager support for vx
//!
//! This provides Bun package manager integration for the vx tool.

use anyhow::Result;
use std::path::Path;
use vx_core::{Ecosystem, PackageSpec, VxPackageManager, VxPlugin, VxTool};

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

/// Bun plugin
#[derive(Default)]
pub struct BunPlugin;

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
        vec![]
    }

    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(BunPackageManager)]
    }

    fn supports_tool(&self, _tool_name: &str) -> bool {
        false
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
        assert_eq!(pm.ecosystem(), Ecosystem::JavaScript);
        assert_eq!(
            pm.description(),
            "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
        );
    }

    #[test]
    fn test_bun_plugin() {
        let plugin = BunPlugin;
        assert_eq!(plugin.name(), "bun");
        assert_eq!(plugin.version(), "1.0.0");
    }
}
