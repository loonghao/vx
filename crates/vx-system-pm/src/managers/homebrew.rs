//! Homebrew package manager implementation

use super::{InstallResult, PackageInstallSpec, SystemPackageManager};
use crate::{Result, SystemPmError};
use async_trait::async_trait;
use std::process::Command;
use tracing::{debug, info, warn};

/// Homebrew package manager
pub struct HomebrewManager;

impl HomebrewManager {
    /// Create a new Homebrew manager
    pub fn new() -> Self {
        Self
    }

    /// Run a brew command
    fn run_brew(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new("brew").args(args).output()
    }

    /// Run a shell command
    #[allow(dead_code)]
    fn run_shell(&self, script: &str) -> std::io::Result<std::process::Output> {
        Command::new("bash").args(["-c", script]).output()
    }
}

impl Default for HomebrewManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemPackageManager for HomebrewManager {
    fn name(&self) -> &str {
        "brew"
    }

    fn supported_platforms(&self) -> Vec<&str> {
        vec!["macos", "linux"]
    }

    async fn is_installed(&self) -> bool {
        which::which("brew").is_ok()
    }

    async fn install_self(&self) -> Result<()> {
        info!("Installing Homebrew...");

        let script = r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#;

        // Note: This requires user interaction, so we provide instructions instead
        Err(SystemPmError::Other(anyhow::anyhow!(
            "Homebrew installation requires user interaction. Please run:\n\n  {}\n\nThen retry the command.",
            script
        )))
    }

    async fn install_package(&self, spec: &PackageInstallSpec) -> Result<InstallResult> {
        if !self.is_installed().await {
            return Err(SystemPmError::PackageManagerNotInstalled(
                "brew".to_string(),
            ));
        }

        let args = vec!["install", &spec.package];

        debug!("Running: brew {}", args.join(" "));

        let output = self.run_brew(&args)?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            info!("Package {} installed successfully", spec.package);

            // Get installed version
            let version = self.get_installed_version(&spec.package).await?;

            Ok(InstallResult::success()
                .with_version(version.unwrap_or_else(|| "unknown".to_string())))
        } else {
            // Check if already installed
            if stderr.contains("already installed") {
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
                "brew".to_string(),
            ));
        }

        let output = self.run_brew(&["uninstall", package])?;

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

        let output = self.run_brew(&["list", package])?;
        Ok(output.status.success())
    }

    async fn get_installed_version(&self, package: &str) -> Result<Option<String>> {
        if !self.is_installed().await {
            return Ok(None);
        }

        let output = self.run_brew(&["info", package, "--json=v2"])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse JSON to get version
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(formulae) = json.get("formulae").and_then(|f| f.as_array()) {
                    if let Some(first) = formulae.first() {
                        if let Some(versions) = first.get("versions") {
                            if let Some(stable) = versions.get("stable").and_then(|v| v.as_str()) {
                                return Ok(Some(stable.to_string()));
                            }
                        }
                    }
                }
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
    fn test_homebrew_manager_creation() {
        let manager = HomebrewManager::new();
        assert_eq!(manager.name(), "brew");
        assert_eq!(manager.supported_platforms(), vec!["macos", "linux"]);
        assert_eq!(manager.priority(), 90);
    }
}
