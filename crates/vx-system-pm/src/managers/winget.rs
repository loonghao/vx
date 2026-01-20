//! winget package manager implementation
//!
//! This module provides winget integration with:
//! - **Silent installation**: Uses `--silent`, `--disable-interactivity` flags
//! - **Non-interactive mode**: Avoids all user prompts in CI/automated environments
//! - **Progress reporting**: Provides status callbacks during installation

use super::{InstallResult, PackageInstallSpec, SystemPackageManager};
use crate::{Result, SystemPmError};
use async_trait::async_trait;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tracing::{debug, info, trace, warn};

/// Progress callback type for installation status updates
pub type ProgressCallback = Box<dyn Fn(&str) + Send + Sync>;

/// winget package manager
pub struct WingetManager {
    /// Optional progress callback
    progress_callback: Option<ProgressCallback>,
}

impl WingetManager {
    /// Create a new winget manager
    pub fn new() -> Self {
        Self {
            progress_callback: None,
        }
    }

    /// Create a winget manager with progress callback
    pub fn with_progress<F>(callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        Self {
            progress_callback: Some(Box::new(callback)),
        }
    }

    /// Report progress through callback
    fn report_progress(&self, message: &str) {
        if let Some(ref callback) = self.progress_callback {
            callback(message);
        }
        trace!("winget progress: {}", message);
    }

    /// Run a winget command (simple, no streaming)
    fn run_winget(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new("winget").args(args).output()
    }

    /// Run a winget command with streaming output for progress tracking
    fn run_winget_with_progress(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        let mut cmd = Command::new("winget");
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Read stdout in a separate thread to provide progress updates
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(|r| r.ok()) {
                // Parse and report progress from winget output
                if !line.trim().is_empty() {
                    self.report_progress(&line);
                }
            }
        }

        child.wait_with_output()
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

        self.report_progress(&format!("Installing {} via winget...", spec.package));

        let mut args = vec![
            "install",
            "--id",
            &spec.package,
            // Essential silent/non-interactive flags
            "--accept-package-agreements",
            "--accept-source-agreements",
            "--disable-interactivity", // Prevent all interactive prompts
        ];

        // Add version if specified
        let version_str;
        if let Some(version) = &spec.version {
            version_str = version.clone();
            args.extend(["--version", &version_str]);
        }

        // Silent mode - use the native installer's silent switches
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
        self.report_progress(&format!("Running: winget {}", args.join(" ")));

        // Run with progress tracking if callback is set
        let output = if self.progress_callback.is_some() {
            self.run_winget_with_progress(&args)?
        } else {
            self.run_winget(&args)?
        };

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
            // Check for specific error codes
            let exit_code = output.status.code().unwrap_or(-1);

            // 0x8A150014 = Package already installed
            if stdout.contains("already installed") || exit_code == -1978335212 {
                info!("Package {} is already installed", spec.package);
                self.report_progress(&format!("{} is already installed", spec.package));
                let version = self.get_installed_version(&spec.package).await?;
                return Ok(InstallResult::success()
                    .with_version(version.unwrap_or_else(|| "unknown".to_string())));
            }

            warn!("Failed to install {}: {}", spec.package, stderr);
            self.report_progress(&format!("Failed to install {}", spec.package));
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

        self.report_progress(&format!("Uninstalling {} via winget...", package));

        // Silent, non-interactive uninstallation
        let output = self.run_winget(&[
            "uninstall",
            "--id",
            package,
            "--silent",
            "--disable-interactivity",
        ])?;

        if output.status.success() {
            info!("Package {} uninstalled successfully", package);
            self.report_progress(&format!("{} uninstalled successfully", package));
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

        let output = self.run_winget(&[
            "list",
            "--id",
            package,
            "--exact",
            "--disable-interactivity",
        ])?;

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

        let output = self.run_winget(&[
            "list",
            "--id",
            package,
            "--exact",
            "--disable-interactivity",
        ])?;

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
        // winget is built-in on Windows 11 and modern Windows 10 (1709+)
        // Prefer winget over choco/scoop as it's the official Microsoft tool
        95
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
        // winget has highest priority on Windows as it's built-in on Win11
        assert_eq!(manager.priority(), 95);
    }

    #[test]
    fn test_winget_manager_with_progress() {
        let messages = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let messages_clone = messages.clone();

        let manager = WingetManager::with_progress(move |msg| {
            messages_clone.lock().unwrap().push(msg.to_string());
        });

        manager.report_progress("test winget message");

        let captured = messages.lock().unwrap();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0], "test winget message");
    }

    #[test]
    fn test_winget_default() {
        let manager = WingetManager::default();
        assert_eq!(manager.name(), "winget");
    }
}
