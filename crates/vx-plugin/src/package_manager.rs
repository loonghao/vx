//! Package manager plugin trait and related functionality
//!
//! This module defines the `VxPackageManager` trait, which provides a unified
//! interface for different package managers across various ecosystems.

use crate::{Ecosystem, IsolationLevel, PackageInfo, PackageManagerConfig, PackageSpec, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Simplified trait for implementing package manager support
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
///
/// # Example
///
/// ```rust,no_run
/// use vx_plugin::{VxPackageManager, Ecosystem, PackageSpec, Result};
/// use async_trait::async_trait;
/// use std::path::Path;
///
/// struct MyPackageManager;
///
/// #[async_trait]
/// impl VxPackageManager for MyPackageManager {
///     fn name(&self) -> &str {
///         "mypm"
///     }
///
///     fn ecosystem(&self) -> Ecosystem {
///         Ecosystem::Node
///     }
///
///     async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
///         // Install packages using your package manager
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait VxPackageManager: Send + Sync {
    /// Package manager name (required)
    ///
    /// This should be the command name used to invoke the package manager,
    /// such as "npm", "pip", "cargo", etc.
    fn name(&self) -> &str;

    /// Ecosystem this package manager belongs to (required)
    ///
    /// Indicates which programming language or platform ecosystem
    /// this package manager serves.
    fn ecosystem(&self) -> Ecosystem;

    /// Description of the package manager (optional)
    ///
    /// A human-readable description of what this package manager does.
    fn description(&self) -> &str {
        "A package manager"
    }

    /// Check if this package manager is available on the system
    ///
    /// Default implementation checks if the executable exists in PATH.
    async fn is_available(&self) -> Result<bool> {
        Ok(which::which(self.name()).is_ok())
    }

    /// Check if this package manager should be used for a project
    ///
    /// Override this to detect project-specific files (package.json, Cargo.toml, etc.)
    /// The default implementation checks for common configuration files.
    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        let config_files = self.get_config_files();
        config_files
            .iter()
            .any(|file| project_path.join(file).exists())
    }

    /// Get configuration files that indicate this package manager should be used
    ///
    /// Override this to return the configuration files specific to your package manager.
    /// For example, npm would return ["package.json"], cargo would return ["Cargo.toml"].
    fn get_config_files(&self) -> Vec<&str> {
        vec![]
    }
    /// Install packages (main method to implement)
    ///
    /// This is the primary method that package manager implementations must provide.
    /// It should install the specified packages in the given project directory.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package specifications to install
    /// * `project_path` - Path to the project directory
    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()>;

    /// Remove packages
    ///
    /// Default implementation uses the "remove" command.
    /// Override if your package manager uses different commands.
    async fn remove_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        self.run_command(&["remove"], packages, project_path).await
    }

    /// Update packages
    ///
    /// Default implementation uses the "update" command.
    /// If no packages are specified, updates all packages.
    async fn update_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        if packages.is_empty() {
            self.run_command(&["update"], &[], project_path).await
        } else {
            self.run_command(&["update"], packages, project_path).await
        }
    }

    /// List installed packages
    ///
    /// Default implementation attempts to parse common configuration files
    /// or run list commands. Override for package manager-specific logic.
    async fn list_packages(&self, project_path: &Path) -> Result<Vec<PackageInfo>> {
        self.default_list_packages(project_path).await
    }

    /// Search for packages
    ///
    /// Default implementation runs the package manager's search command.
    /// Override for custom search logic or API integration.
    async fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>> {
        self.run_search_command(query).await
    }
    /// Run a package manager command with arguments
    ///
    /// This is a utility method that executes the package manager with
    /// the specified command and arguments in the given project directory.
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
    ///
    /// This method provides a fallback implementation that attempts
    /// to parse common configuration files. Override for better integration.
    async fn default_list_packages(&self, _project_path: &Path) -> Result<Vec<PackageInfo>> {
        // Default implementation returns empty list
        // Real implementations would parse lock files, run list commands, etc.
        Ok(vec![])
    }

    /// Default implementation for searching packages
    ///
    /// This method runs the package manager's search command and attempts
    /// to parse the output. Override for API-based search or custom parsing.
    async fn run_search_command(&self, _query: &str) -> Result<Vec<PackageInfo>> {
        // Default implementation returns empty list
        // Real implementations would run search commands and parse output
        Ok(vec![])
    }
    /// Get the command to install packages
    ///
    /// Override this if your package manager uses a different command for installation.
    /// Most package managers use "install", but some might use "add" or other commands.
    fn get_install_command(&self) -> Vec<&str> {
        vec!["install"]
    }

    /// Get the command to add new packages
    ///
    /// Override this if your package manager distinguishes between installing
    /// existing dependencies and adding new ones. Some package managers use "add"
    /// for new packages and "install" for existing dependencies.
    fn get_add_command(&self) -> Vec<&str> {
        vec!["add"]
    }

    /// Get the command to remove packages
    ///
    /// Override this if your package manager uses a different command for removal.
    fn get_remove_command(&self) -> Vec<&str> {
        vec!["remove"]
    }

    /// Get the command to update packages
    ///
    /// Override this if your package manager uses a different command for updates.
    fn get_update_command(&self) -> Vec<&str> {
        vec!["update"]
    }

    /// Get the command to list packages
    ///
    /// Override this if your package manager has a specific list command.
    fn get_list_command(&self) -> Vec<&str> {
        vec!["list"]
    }

    /// Get the command to search packages
    ///
    /// Override this if your package manager uses a different search command.
    fn get_search_command(&self) -> Vec<&str> {
        vec!["search"]
    }

    /// Get package manager configuration
    ///
    /// Returns configuration information about this package manager.
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
    ///
    /// Similar to run_command but returns the exit code instead of failing on non-zero codes.
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

    /// Additional metadata for the package manager (optional)
    ///
    /// Override this to provide package manager-specific metadata such as
    /// supported features, configuration options, etc.
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
        let name = name.into();
        Self {
            name: name.clone(),
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
impl VxPackageManager for StandardPackageManager {
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
