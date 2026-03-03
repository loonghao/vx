//! APT package manager implementation

use super::{
    InstallResult, PackageInstallSpec, ProgressCallback, SystemPackageManager,
    run_command_with_progress,
};
use crate::{Result, SystemPmError};
use async_trait::async_trait;
use std::process::Command;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// APT package manager (Debian/Ubuntu)
pub struct AptManager {
    /// Optional progress callback
    progress_callback: Option<ProgressCallback>,
}

impl AptManager {
    /// Create a new APT manager
    pub fn new() -> Self {
        Self {
            progress_callback: None,
        }
    }

    /// Create an APT manager with progress callback
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

    /// Check if running as root
    fn is_root() -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }
        #[cfg(not(unix))]
        {
            false
        }
    }

    /// Run an apt command (with sudo if needed)
    fn run_apt(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        if Self::is_root() {
            Command::new("apt-get").args(args).output()
        } else {
            let mut cmd_args = vec!["apt-get"];
            cmd_args.extend(args);
            Command::new("sudo").args(cmd_args).output()
        }
    }

    /// Run an apt command with streaming progress output
    fn run_apt_with_progress(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        let mut cmd = if Self::is_root() {
            let mut c = Command::new("apt-get");
            c.args(args);
            c
        } else {
            let mut c = Command::new("sudo");
            c.arg("apt-get").args(args);
            c
        };

        if let Some(callback) = &self.progress_callback {
            run_command_with_progress(cmd, callback)
        } else {
            cmd.output()
        }
    }

    /// Run dpkg-query
    fn run_dpkg_query(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new("dpkg-query").args(args).output()
    }
}

impl Default for AptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemPackageManager for AptManager {
    fn name(&self) -> &str {
        "apt"
    }

    fn supported_platforms(&self) -> Vec<&str> {
        vec!["linux"]
    }

    async fn is_installed(&self) -> bool {
        which::which("apt-get").is_ok()
    }

    async fn install_self(&self) -> Result<()> {
        // APT is the default package manager on Debian/Ubuntu
        // If not available, this is not a Debian-based system
        Err(SystemPmError::Other(anyhow::anyhow!(
            "APT is not available. This system is not Debian/Ubuntu based."
        )))
    }

    async fn install_package(&self, spec: &PackageInstallSpec) -> Result<InstallResult> {
        if !self.is_installed().await {
            return Err(SystemPmError::PackageManagerNotInstalled("apt".to_string()));
        }

        // Update package list first
        debug!("Updating package list...");
        self.report_progress("Updating package list...");
        let _ = self.run_apt(&["update", "-qq"]);

        let args = vec!["install", "-y", "-qq", &spec.package];

        debug!("Running: apt-get {}", args.join(" "));
        self.report_progress(&format!("Installing {} via apt-get...", spec.package));

        let output = self.run_apt_with_progress(&args)?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            info!("Package {} installed successfully", spec.package);
            self.report_progress(&format!("{} installed successfully", spec.package));

            // Get installed version
            let version = self.get_installed_version(&spec.package).await?;

            Ok(InstallResult::success()
                .with_version(version.unwrap_or_else(|| "unknown".to_string())))
        } else {
            warn!("Failed to install {}: {}", spec.package, stderr);
            Err(SystemPmError::InstallationFailed {
                package: spec.package.clone(),
                reason: format!("{}\n{}", stdout, stderr),
            })
        }
    }

    async fn uninstall_package(&self, package: &str) -> Result<()> {
        if !self.is_installed().await {
            return Err(SystemPmError::PackageManagerNotInstalled("apt".to_string()));
        }

        let output = self.run_apt(&["remove", "-y", "-qq", package])?;

        if output.status.success() {
            info!("Package {} uninstalled successfully", package);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(SystemPmError::CommandFailed(format!(
                "Failed to uninstall {}: {}",
                package, stderr
            )))
        }
    }

    async fn is_package_installed(&self, package: &str) -> Result<bool> {
        if !self.is_installed().await {
            return Ok(false);
        }

        let output = self.run_dpkg_query(&["-W", "-f=${Status}", package])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.contains("install ok installed"))
        } else {
            Ok(false)
        }
    }

    async fn get_installed_version(&self, package: &str) -> Result<Option<String>> {
        if !self.is_installed().await {
            return Ok(None);
        }

        let output = self.run_dpkg_query(&["-W", "-f=${Version}", package])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout.trim();
            if !version.is_empty() {
                return Ok(Some(version.to_string()));
            }
        }

        Ok(None)
    }

    fn priority(&self) -> i32 {
        90
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apt_manager_creation() {
        let manager = AptManager::new();
        assert_eq!(manager.name(), "apt");
        assert_eq!(manager.supported_platforms(), vec!["linux"]);
        assert_eq!(manager.priority(), 90);
    }
}
