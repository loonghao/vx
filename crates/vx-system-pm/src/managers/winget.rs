//! winget package manager implementation

use super::{InstallResult, PackageInstallSpec, SystemPackageManager};
use crate::{Result, SystemPmError};
use async_trait::async_trait;
use std::process::Command;
use tracing::{debug, info, warn};

/// winget package manager
pub struct WingetManager;

impl WingetManager {
    /// Create a new winget manager
    pub fn new() -> Self {
        Self
    }

    /// Run a winget command
    fn run_winget(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new("winget").args(args).output()
    }
}

impl Default for WingetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemPackageManager for WingetManager {
    fn name(&self) -> &str {
        "winget"
    }

    fn supported_platforms(&self) -> Vec<&str> {
        vec!["windows"]
    }

    async fn is_installed(&self) -> bool {
        which::which("winget").is_ok()
    }

    async fn install_self(&self) -> Result<()> {
        // winget comes pre-installed on Windows 10 1709+ and Windows 11
        // If not available, user needs to install App Installer from Microsoft Store
        Err(SystemPmError::Other(anyhow::anyhow!(
            "winget is not installed. Please install 'App Installer' from the Microsoft Store, \
             or use Windows Update to get the latest version."
        )))
    }

    async fn install_package(&self, spec: &PackageInstallSpec) -> Result<InstallResult> {
        if !self.is_installed().await {
            return Err(SystemPmError::PackageManagerNotInstalled(
                "winget".to_string(),
            ));
        }

        let mut args = vec![
            "install",
            "--id",
            &spec.package,
            "--accept-package-agreements",
            "--accept-source-agreements",
        ];

        // Add version if specified
        let version_str;
        if let Some(version) = &spec.version {
            version_str = version.clone();
            args.extend(["--version", &version_str]);
        }

        // Silent mode
        if spec.silent {
            args.push("--silent");
        }

        // Install location
        let location_str;
        if let Some(dir) = &spec.install_dir {
            location_str = dir.to_string_lossy().to_string();
            args.extend(["--location", &location_str]);
        }

        debug!("Running: winget {}", args.join(" "));

        let output = self.run_winget(&args)?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            info!("Package {} installed successfully", spec.package);

            // Get installed version
            let version = self.get_installed_version(&spec.package).await?;

            Ok(InstallResult::success()
                .with_version(version.unwrap_or_else(|| "unknown".to_string())))
        } else {
            // Check for specific error codes
            let exit_code = output.status.code().unwrap_or(-1);

            // 0x8A150014 = Package already installed
            if stdout.contains("already installed") || exit_code == -1978335212 {
                info!("Package {} is already installed", spec.package);
                let version = self.get_installed_version(&spec.package).await?;
                return Ok(InstallResult::success()
                    .with_version(version.unwrap_or_else(|| "unknown".to_string())));
            }

            warn!("Failed to install {}: {}", spec.package, stderr);
            Err(SystemPmError::InstallationFailed {
                package: spec.package.clone(),
                reason: format!("{}\n{}", stdout, stderr),
            })
        }
    }

    async fn uninstall_package(&self, package: &str) -> Result<()> {
        if !self.is_installed().await {
            return Err(SystemPmError::PackageManagerNotInstalled(
                "winget".to_string(),
            ));
        }

        let output = self.run_winget(&["uninstall", "--id", package, "--silent"])?;

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

        let output = self.run_winget(&["list", "--id", package, "--exact"])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Check if the package appears in the output
            Ok(stdout.contains(package))
        } else {
            Ok(false)
        }
    }

    async fn get_installed_version(&self, package: &str) -> Result<Option<String>> {
        if !self.is_installed().await {
            return Ok(None);
        }

        let output = self.run_winget(&["list", "--id", package, "--exact"])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse winget list output
            // Format: Name  Id  Version  Available  Source
            for line in stdout.lines() {
                if line.contains(package) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    // Version is usually the 3rd column
                    if parts.len() >= 3 {
                        // Find the version (looks like x.y.z)
                        for part in &parts[1..] {
                            if part.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                                return Ok(Some(part.to_string()));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn priority(&self) -> i32 {
        70
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winget_manager_creation() {
        let manager = WingetManager::new();
        assert_eq!(manager.name(), "winget");
        assert_eq!(manager.supported_platforms(), vec!["windows"]);
        assert_eq!(manager.priority(), 70);
    }
}
