//! Scoop package manager implementation

use super::{
    InstallResult, PackageInstallSpec, ProgressCallback, SystemPackageManager,
    run_command_with_progress,
};
use crate::{Result, SystemPmError};
use async_trait::async_trait;
use std::process::Command;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Scoop package manager (Windows)
pub struct ScoopManager {
    /// Optional progress callback
    progress_callback: Option<ProgressCallback>,
}

impl ScoopManager {
    /// Create a new Scoop manager
    pub fn new() -> Self {
        Self {
            progress_callback: None,
        }
    }

    /// Create a Scoop manager with progress callback
    pub fn with_progress<F>(callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        Self {
            progress_callback: Some(Arc::new(callback)),
        }
    }

    /// Report progress through callback
    fn report_progress(&self, message: &str) {
        if let Some(ref callback) = self.progress_callback {
            callback(message);
        }
    }

    /// Get the scoop executable path
    fn scoop_path() -> Option<std::path::PathBuf> {
        which::which("scoop").ok()
    }

    /// Run a scoop command with streaming progress output
    fn run_scoop_with_progress(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        let mut cmd = Command::new("scoop");
        cmd.args(args);
        if let Some(callback) = &self.progress_callback {
            run_command_with_progress(cmd, callback)
        } else {
            cmd.output()
        }
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
        self.report_progress(&format!("Installing {} via Scoop...", spec.package));

        let package_arg = if let Some(version) = &spec.version {
            // Try bucket/package@version format
            format!("{}@{}", spec.package, version)
        } else {
            spec.package.clone()
        };
        let output = self.run_scoop_with_progress(&["install", package_arg.as_str()])?;

        let _stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            info!("Package {} installed successfully via Scoop", spec.package);
            self.report_progress(&format!("{} installed successfully", spec.package));
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
        60 // Lower than winget (95) and choco (80)
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
