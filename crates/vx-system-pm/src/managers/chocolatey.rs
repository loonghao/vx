//! Chocolatey package manager implementation

use super::{InstallResult, PackageInstallSpec, SystemPackageManager};
use crate::{Result, SystemPmError};
use async_trait::async_trait;
use std::process::Command;
use tracing::{debug, info, warn};

/// Chocolatey package manager
pub struct ChocolateyManager;

impl ChocolateyManager {
    /// Create a new Chocolatey manager
    pub fn new() -> Self {
        Self
    }

    /// Check if running with administrator privileges
    fn is_elevated() -> bool {
        crate::detector::is_elevated()
    }

    /// Run a choco command
    fn run_choco(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new("choco").args(args).output()
    }

    /// Run a PowerShell command
    fn run_powershell(&self, script: &str) -> std::io::Result<std::process::Output> {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script,
            ])
            .output()
    }
}

impl Default for ChocolateyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemPackageManager for ChocolateyManager {
    fn name(&self) -> &str {
        "choco"
    }

    fn supported_platforms(&self) -> Vec<&str> {
        vec!["windows"]
    }

    async fn is_installed(&self) -> bool {
        which::which("choco").is_ok()
    }

    async fn install_self(&self) -> Result<()> {
        if !Self::is_elevated() {
            return Err(SystemPmError::ElevationRequired(
                "Administrator privileges required to install Chocolatey".to_string(),
            ));
        }

        info!("Installing Chocolatey...");

        let script = r#"
            Set-ExecutionPolicy Bypass -Scope Process -Force;
            [System.Net.ServicePointManager]::SecurityProtocol = 
                [System.Net.ServicePointManager]::SecurityProtocol -bor 3072;
            iex ((New-Object System.Net.WebClient).DownloadString(
                'https://community.chocolatey.org/install.ps1'))
        "#;

        let output = self.run_powershell(script)?;

        if output.status.success() {
            info!("Chocolatey installed successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(SystemPmError::InstallationFailed {
                package: "chocolatey".to_string(),
                reason: stderr.to_string(),
            })
        }
    }

    async fn install_package(&self, spec: &PackageInstallSpec) -> Result<InstallResult> {
        if !self.is_installed().await {
            return Err(SystemPmError::PackageManagerNotInstalled(
                "choco".to_string(),
            ));
        }

        let mut args = vec!["install", &spec.package, "-y"];

        // Add version if specified
        let version_str;
        if let Some(version) = &spec.version {
            version_str = version.clone();
            args.extend(["--version", &version_str]);
        }

        // Add params if specified
        let params_str;
        if let Some(params) = &spec.params {
            params_str = format!("\"{}\"", params);
            args.extend(["--params", &params_str]);
        }

        // Add install-arguments if specified
        let install_args_str;
        if let Some(install_args) = &spec.install_args {
            install_args_str = format!("\"{}\"", install_args);
            args.extend(["--install-arguments", &install_args_str]);
        }

        debug!("Running: choco {}", args.join(" "));

        let output = self.run_choco(&args)?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            info!("Package {} installed successfully", spec.package);

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
            return Err(SystemPmError::PackageManagerNotInstalled(
                "choco".to_string(),
            ));
        }

        let output = self.run_choco(&["uninstall", package, "-y"])?;

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

        let output = self.run_choco(&["list", "--local-only", "--exact", package])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Chocolatey output format: "package version"
            Ok(stdout.lines().any(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                parts
                    .first()
                    .is_some_and(|&name| name.eq_ignore_ascii_case(package))
            }))
        } else {
            Ok(false)
        }
    }

    async fn get_installed_version(&self, package: &str) -> Result<Option<String>> {
        if !self.is_installed().await {
            return Ok(None);
        }

        let output = self.run_choco(&["list", "--local-only", "--exact", package])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0].eq_ignore_ascii_case(package) {
                    return Ok(Some(parts[1].to_string()));
                }
            }
        }

        Ok(None)
    }

    fn priority(&self) -> i32 {
        80
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chocolatey_manager_creation() {
        let manager = ChocolateyManager::new();
        assert_eq!(manager.name(), "choco");
        assert_eq!(manager.supported_platforms(), vec!["windows"]);
    }

    #[test]
    fn test_package_install_spec() {
        let spec = PackageInstallSpec::new("git")
            .with_params("/GitAndUnixToolsOnPath")
            .with_install_args("/DIR=C:\\git");

        assert_eq!(spec.package, "git");
        assert_eq!(spec.params, Some("/GitAndUnixToolsOnPath".to_string()));
        assert_eq!(spec.install_args, Some("/DIR=C:\\git".to_string()));
    }
}
