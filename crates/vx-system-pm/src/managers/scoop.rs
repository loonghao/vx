//! Scoop package manager implementation

use super::{InstallResult, PackageInstallSpec, SystemPackageManager};
use crate::{Result, SystemPmError};
use async_trait::async_trait;
use std::process::Command;
use tracing::{debug, info, warn};

/// Scoop package manager (Windows)
pub struct ScoopManager;

impl ScoopManager {
    /// Create a new Scoop manager
    pub fn new() -> Self {
        Self
    }

    /// Get the scoop executable path
    fn scoop_path() -> Option<std::path::PathBuf> {
        which::which("scoop").ok()
    }
}

impl Default for ScoopManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemPackageManager for ScoopManager {
    fn name(&self) -> &str {
        "scoop"
    }

    fn supported_platforms(&self) -> Vec<&str> {
        vec!["windows"]
    }

    async fn is_installed(&self) -> bool {
        Self::scoop_path().is_some()
    }

    async fn install_self(&self) -> Result<()> {
        info!("Installing Scoop...");

        // Scoop installation requires PowerShell
        let script = r#"
            Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force
            Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression
        "#;

        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script,
            ])
            .status()?;

        if status.success() {
            info!("Scoop installed successfully");
            Ok(())
        } else {
            Err(SystemPmError::InstallFailed(
                "Failed to install Scoop".to_string(),
            ))
        }
    }

    async fn install_package(&self, spec: &PackageInstallSpec) -> Result<InstallResult> {
        debug!("Installing package via Scoop: {}", spec.package);

        let mut args = vec!["install".to_string(), spec.package.clone()];

        // Scoop doesn't support version pinning in the same way as other package managers
        // but we can try to install a specific version if available
        if let Some(version) = &spec.version {
            // Try bucket/package@version format
            args[1] = format!("{}@{}", spec.package, version);
        }

        let output = Command::new("scoop").args(&args).output()?;

        let _stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            let version = self.get_installed_version(&spec.package).await?;
            Ok(InstallResult::success()
                .with_version(version.unwrap_or_else(|| "unknown".to_string())))
        } else {
            warn!("Scoop install failed: {}", stderr);
            Err(SystemPmError::InstallFailed(format!(
                "Scoop install failed: {}",
                stderr
            )))
        }
    }

    async fn uninstall_package(&self, package: &str) -> Result<()> {
        debug!("Uninstalling package via Scoop: {}", package);

        let status = Command::new("scoop")
            .args(["uninstall", package])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(SystemPmError::UninstallFailed(format!(
                "Failed to uninstall {}",
                package
            )))
        }
    }

    async fn is_package_installed(&self, package: &str) -> Result<bool> {
        let output = Command::new("scoop").args(["list", package]).output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Check if the package is in the output
            Ok(stdout.lines().any(|line| {
                line.split_whitespace()
                    .next()
                    .map(|name| name == package)
                    .unwrap_or(false)
            }))
        } else {
            Ok(false)
        }
    }

    async fn get_installed_version(&self, package: &str) -> Result<Option<String>> {
        let output = Command::new("scoop").args(["list", package]).output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse output: "package version source"
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == package {
                    return Ok(Some(parts[1].to_string()));
                }
            }
        }
        Ok(None)
    }

    fn priority(&self) -> i32 {
        60 // Lower than choco (80) and winget (70)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoop_manager_creation() {
        let manager = ScoopManager::new();
        assert_eq!(manager.name(), "scoop");
        assert_eq!(manager.supported_platforms(), vec!["windows"]);
    }

    #[test]
    fn test_scoop_priority() {
        let manager = ScoopManager::new();
        assert_eq!(manager.priority(), 60);
    }
}
