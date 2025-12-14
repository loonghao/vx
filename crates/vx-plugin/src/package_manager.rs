//! Package manager trait and related functionality
//!
//! This module defines the `PackageManager` trait (formerly `VxPackageManager`),
//! which provides a unified interface for different package managers across various ecosystems.

use crate::{Ecosystem, IsolationLevel, PackageInfo, PackageManagerConfig, PackageSpec, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Package manager trait for unified package management
///
/// This trait provides a high-level interface for package managers,
/// with sensible defaults for common operations.
///
/// # Required Methods
///
/// - `name()`: Return the package manager name
/// - `ecosystem()`: Return the ecosystem this package manager belongs to
/// - `install_packages()`: Install packages in a project
///
/// # Optional Methods
///
/// All other methods have default implementations, but can be overridden
/// for package manager-specific behavior.
#[async_trait]
pub trait PackageManager: Send + Sync {
    /// Package manager name (required)
    fn name(&self) -> &str;

    /// Ecosystem this package manager belongs to (required)
    fn ecosystem(&self) -> Ecosystem;

    /// Description of the package manager (optional)
    fn description(&self) -> &str {
        "A package manager"
    }

    /// Check if this package manager is available on the system
    async fn is_available(&self) -> Result<bool> {
        Ok(which::which(self.name()).is_ok())
    }

    /// Check if this package manager should be used for a project
    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        let config_files = self.get_config_files();
        config_files
            .iter()
            .any(|file| project_path.join(file).exists())
    }

    /// Get configuration files that indicate this package manager should be used
    fn get_config_files(&self) -> Vec<&str> {
        vec![]
    }

    /// Install packages (main method to implement)
    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()>;

    /// Remove packages
    async fn remove_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        self.run_command(&["remove"], packages, project_path).await
    }

    /// Update packages
    async fn update_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        if packages.is_empty() {
            self.run_command(&["update"], &[], project_path).await
        } else {
            self.run_command(&["update"], packages, project_path).await
        }
    }

    /// List installed packages
    async fn list_packages(&self, project_path: &Path) -> Result<Vec<PackageInfo>> {
        self.default_list_packages(project_path).await
    }

    /// Search for packages
    async fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>> {
        self.run_search_command(query).await
    }

    /// Run a package manager command with arguments
    async fn run_command(
        &self,
        command: &[&str],
        args: &[String],
        project_path: &Path,
    ) -> Result<()> {
        let mut cmd = std::process::Command::new(self.name());
        cmd.args(command);
        cmd.args(args);
        cmd.current_dir(project_path);

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run {} command: {}", self.name(), e))?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "{} command failed with exit code: {:?}",
                self.name(),
                status.code()
            ));
        }

        Ok(())
    }

    /// Default implementation for listing packages
    async fn default_list_packages(&self, _project_path: &Path) -> Result<Vec<PackageInfo>> {
        Ok(vec![])
    }

    /// Default implementation for searching packages
    async fn run_search_command(&self, _query: &str) -> Result<Vec<PackageInfo>> {
        Ok(vec![])
    }

    /// Get the command to install packages
    fn get_install_command(&self) -> Vec<&str> {
        vec!["install"]
    }

    /// Get the command to add new packages
    fn get_add_command(&self) -> Vec<&str> {
        vec!["add"]
    }

    /// Get the command to remove packages
    fn get_remove_command(&self) -> Vec<&str> {
        vec!["remove"]
    }

    /// Get the command to update packages
    fn get_update_command(&self) -> Vec<&str> {
        vec!["update"]
    }

    /// Get the command to list packages
    fn get_list_command(&self) -> Vec<&str> {
        vec!["list"]
    }

    /// Get the command to search packages
    fn get_search_command(&self) -> Vec<&str> {
        vec!["search"]
    }

    /// Get package manager configuration
    fn get_config(&self) -> PackageManagerConfig {
        PackageManagerConfig {
            name: self.name().to_string(),
            version: None,
            executable_path: which::which(self.name()).ok(),
            config_files: self.get_config_files().iter().map(PathBuf::from).collect(),
            cache_directory: None,
            supports_lockfiles: true,
            supports_workspaces: false,
            isolation_level: IsolationLevel::Project,
        }
    }

    /// Run a package manager command and return the exit code
    async fn run_command_with_code(
        &self,
        command: &[&str],
        args: &[String],
        project_path: &Path,
    ) -> Result<i32> {
        let mut cmd = std::process::Command::new(self.name());
        cmd.args(command);
        cmd.args(args);
        cmd.current_dir(project_path);

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run {} command: {}", self.name(), e))?;

        Ok(status.code().unwrap_or(-1))
    }

    /// Additional metadata for the package manager
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Standard package manager implementation
///
/// This is a convenience implementation for package managers that follow
/// common patterns. It provides sensible defaults and can be customized
/// through configuration.
pub struct StandardPackageManager {
    name: String,
    description: String,
    ecosystem: Ecosystem,
    config_files: Vec<String>,
    install_command: Vec<String>,
    remove_command: Vec<String>,
    update_command: Vec<String>,
    list_command: Vec<String>,
    search_command: Vec<String>,
}

impl StandardPackageManager {
    /// Create a new standard package manager
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        ecosystem: Ecosystem,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            ecosystem,
            config_files: Vec::new(),
            install_command: vec!["install".to_string()],
            remove_command: vec!["remove".to_string()],
            update_command: vec!["update".to_string()],
            list_command: vec!["list".to_string()],
            search_command: vec!["search".to_string()],
        }
    }

    /// Add a configuration file that indicates this package manager should be used
    pub fn with_config_file(mut self, config_file: impl Into<String>) -> Self {
        self.config_files.push(config_file.into());
        self
    }

    /// Set custom install command
    pub fn with_install_command(mut self, command: Vec<String>) -> Self {
        self.install_command = command;
        self
    }

    /// Set custom remove command
    pub fn with_remove_command(mut self, command: Vec<String>) -> Self {
        self.remove_command = command;
        self
    }

    /// Set custom update command
    pub fn with_update_command(mut self, command: Vec<String>) -> Self {
        self.update_command = command;
        self
    }
}

#[async_trait]
impl PackageManager for StandardPackageManager {
    fn name(&self) -> &str {
        &self.name
    }

    fn ecosystem(&self) -> Ecosystem {
        self.ecosystem
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn get_config_files(&self) -> Vec<&str> {
        self.config_files.iter().map(|s| s.as_str()).collect()
    }

    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
        let package_names: Vec<String> = packages
            .iter()
            .map(|p| {
                if let Some(version) = &p.version {
                    format!("{}@{}", p.name, version)
                } else {
                    p.name.clone()
                }
            })
            .collect();

        let command_strs: Vec<&str> = self.install_command.iter().map(|s| s.as_str()).collect();
        self.run_command(&command_strs, &package_names, project_path)
            .await
    }

    fn get_install_command(&self) -> Vec<&str> {
        self.install_command.iter().map(|s| s.as_str()).collect()
    }

    fn get_remove_command(&self) -> Vec<&str> {
        self.remove_command.iter().map(|s| s.as_str()).collect()
    }

    fn get_update_command(&self) -> Vec<&str> {
        self.update_command.iter().map(|s| s.as_str()).collect()
    }

    fn get_list_command(&self) -> Vec<&str> {
        self.list_command.iter().map(|s| s.as_str()).collect()
    }

    fn get_search_command(&self) -> Vec<&str> {
        self.search_command.iter().map(|s| s.as_str()).collect()
    }
}
