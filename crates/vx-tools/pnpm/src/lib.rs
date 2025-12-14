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
}

/// PNPM plugin
#[derive(Default)]
pub struct PnpmPlugin;

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
}
