//! Trait definitions for ecosystem package installers

use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Trait for ecosystem-specific package installers
///
/// This trait defines the interface for installing packages from development
/// ecosystem package managers (npm, pip, cargo, uv, bun, go, gem) with isolation support.
///
/// These installers:
/// - Install to isolated directories (`~/.vx/packages/{ecosystem}/{package}/{version}/`)
/// - Support executable detection for shim creation
/// - Use environment variable redirection for isolation
#[async_trait]
pub trait EcosystemInstaller: Send + Sync {
    /// Get the ecosystem name (npm, pip, cargo, uv, bun, go, gem)
    fn ecosystem(&self) -> &'static str;

    /// Install a package to an isolated directory
    ///
    /// # Arguments
    /// * `install_dir` - The directory to install the package to
    /// * `package` - The package name
    /// * `version` - The version to install ("latest" for latest version)
    /// * `options` - Installation options
    ///
    /// # Returns
    /// Installation result with detected executables
    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult>;

    /// Detect executables in the installed package directory
    ///
    /// # Arguments
    /// * `bin_dir` - The bin directory to scan for executables
    ///
    /// # Returns
    /// List of executable names (without extensions on Windows)
    fn detect_executables(&self, bin_dir: &Path) -> Result<Vec<String>>;

    /// Build environment variables for installation
    ///
    /// These environment variables redirect the package manager to install
    /// to the isolated directory instead of the global location.
    ///
    /// # Arguments
    /// * `install_dir` - The installation directory
    ///
    /// # Returns
    /// Environment configuration for the installation
    fn build_install_env(&self, install_dir: &Path) -> InstallEnv;

    /// Get the bin directory path for a package installation
    ///
    /// # Arguments
    /// * `install_dir` - The installation directory
    ///
    /// # Returns
    /// Path to the bin directory containing executables
    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf;

    /// Uninstall a package by removing its directory
    ///
    /// # Arguments
    /// * `install_dir` - The installation directory to remove
    fn uninstall(&self, install_dir: &Path) -> Result<()> {
        if install_dir.exists() {
            std::fs::remove_dir_all(install_dir)
                .with_context(|| format!("Failed to remove {}", install_dir.display()))?;
        }
        Ok(())
    }

    /// Check if the package manager is available
    fn is_available(&self) -> bool;
}
