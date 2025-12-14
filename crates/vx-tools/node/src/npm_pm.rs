//! NPM package manager implementation
//!
//! This module provides NPM package manager support as part of the Node.js bundle.

use anyhow::Result;
use std::path::Path;
use vx_plugin::{Ecosystem, PackageManager, PackageSpec};

/// NPM package manager implementation
///
/// NPM (Node Package Manager) is the default package manager for Node.js.
/// It is bundled with Node.js and provides dependency management for JavaScript projects.
#[derive(Debug, Default, Clone)]
pub struct NpmPackageManager;

impl NpmPackageManager {
    /// Create a new NPM package manager instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl PackageManager for NpmPackageManager {
    fn name(&self) -> &str {
        "npm"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }

    fn description(&self) -> &str {
        "Node Package Manager - the default package manager for Node.js"
    }

    /// Detect if this is an npm project by looking for package.json
    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        // NPM is preferred if package.json exists and no other lock files are present
        let has_package_json = project_path.join("package.json").exists();
        let has_npm_lock = project_path.join("package-lock.json").exists();
        let has_yarn_lock = project_path.join("yarn.lock").exists();
        let has_pnpm_lock = project_path.join("pnpm-lock.yaml").exists();
        let has_bun_lock = project_path.join("bun.lockb").exists();

        // Prefer npm if it has package-lock.json, or if no other lock files exist
        has_package_json && (has_npm_lock || (!has_yarn_lock && !has_pnpm_lock && !has_bun_lock))
    }

    fn get_config_files(&self) -> Vec<&str> {
        vec!["package.json", "package-lock.json", ".npmrc"]
    }

    /// Install packages using npm
    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
        if packages.is_empty() {
            // Just run `npm install` to install from package.json
            self.run_command(&["install"], &[], project_path).await
        } else {
            // Install specific packages
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
                vec!["install", "--save-dev"]
            } else {
                vec!["install", "--save"]
            };

            self.run_command(&command, &package_names, project_path)
                .await
        }
    }

    /// Remove packages using npm
    async fn remove_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        self.run_command(&["uninstall"], packages, project_path)
            .await
    }

    /// Update packages using npm
    async fn update_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        if packages.is_empty() {
            self.run_command(&["update"], &[], project_path).await
        } else {
            self.run_command(&["update"], packages, project_path).await
        }
    }
}
