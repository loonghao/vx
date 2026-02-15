//! Chocolatey package manager implementation
//!
//! This module provides Chocolatey integration with:
//! - **Silent installation**: Uses `--yes`, `--no-progress`, and `--limit-output` flags
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

/// Chocolatey package manager
pub struct ChocolateyManager {
    /// Optional progress callback
    progress_callback: Option<ProgressCallback>,
}

impl ChocolateyManager {
    /// Create a new Chocolatey manager
    pub fn new() -> Self {
        Self {
            progress_callback: None,
        }
    }

    /// Create a Chocolatey manager with progress callback
    pub fn with_progress<F>(callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        Self {
            progress_callback: Some(Box::new(callback)),
        }
    }

    /// Check if running with administrator privileges
    fn is_elevated() -> bool {
        crate::detector::is_elevated()
    }

    /// Report progress through callback
    fn report_progress(&self, message: &str) {
        if let Some(ref callback) = self.progress_callback {
            callback(message);
        }
        trace!("choco progress: {}", message);
    }

    /// Run a choco command (simple, no streaming)
    fn run_choco(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new("choco").args(args).output()
    }

    /// Run a choco command with streaming output for progress tracking
    fn run_choco_with_progress(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        let mut cmd = Command::new("choco");
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Read stdout in a separate thread to provide progress updates
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(|r| r.ok()) {
                // Parse and report progress from choco output
                if !line.trim().is_empty() {
                    self.report_progress(&line);
                }
            }
        }

        child.wait_with_output()
    }

    /// Run a PowerShell command
    fn run_powershell(&self, script: &str) -> std::io::Result<std::process::Output> {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script,
            ])
            .output()
    }

    /// Build silent installation arguments for choco
    ///
    /// Adds the following flags to enable non-interactive installation:
    /// - `-y`: Confirm all prompts automatically
    /// - `--no-progress`: Disable progress bars for cleaner output
    /// - `--limit-output`: Limit output to essential information
    pub fn build_silent_args<'a>(&self, base_args: Vec<&'a str>) -> Vec<&'a str> {
        let mut args = base_args;
        // Core silent flags
        args.push("-y"); // Confirm all prompts
        args.push("--no-progress"); // Disable progress bars (cleaner output)
        args.push("--limit-output"); // Limit output to essential info
        args
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
        self.report_progress("Installing Chocolatey package manager...");

        // Silent, non-interactive installation
        let script = r#"
            Set-ExecutionPolicy Bypass -Scope Process -Force;
            [System.Net.ServicePointManager]::SecurityProtocol =
                [System.Net.ServicePointManager]::SecurityProtocol -bor 3072;
            $ProgressPreference = 'SilentlyContinue';
            iex ((New-Object System.Net.WebClient).DownloadString(
                'https://community.chocolatey.org/install.ps1'))
        "#;

        let output = self.run_powershell(script)?;

        if output.status.success() {
            info!("Chocolatey installed successfully");
            self.report_progress("Chocolatey installed successfully");
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

        self.report_progress(&format!("Installing {} via Chocolatey...", spec.package));

        // Build base arguments
        let mut base_args = vec!["install", &spec.package];

        // Add version if specified
        let version_str;
        if let Some(version) = &spec.version {
            version_str = version.clone();
            base_args.extend(["--version", &version_str]);
        }

        // Add params if specified
        let params_str;
        if let Some(params) = &spec.params {
            params_str = format!("\"{}\"", params);
            base_args.extend(["--params", &params_str]);
        }

        // Add install-arguments if specified (for native installer silent switches)
        let install_args_str;
        if let Some(install_args) = &spec.install_args {
            install_args_str = format!("\"{}\"", install_args);
            base_args.extend(["--install-arguments", &install_args_str]);
        }

        // Build silent arguments
        let args = self.build_silent_args(base_args);

        debug!("Running: choco {}", args.join(" "));
        self.report_progress(&format!("Running: choco {}", args.join(" ")));

        // Run with progress tracking if callback is set
        let output = if self.progress_callback.is_some() {
            self.run_choco_with_progress(&args)?
        } else {
            self.run_choco(&args)?
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
                "choco".to_string(),
            ));
        }

        self.report_progress(&format!("Uninstalling {} via Chocolatey...", package));

        // Silent uninstallation
        let output = self.run_choco(&["uninstall", package, "-y", "--no-progress"])?;

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

        let output =
            self.run_choco(&["list", "--local-only", "--exact", package, "--limit-output"])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Chocolatey output format: "package|version"
            Ok(stdout.lines().any(|line| {
                let parts: Vec<&str> = line.split('|').collect();
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

        let output =
            self.run_choco(&["list", "--local-only", "--exact", package, "--limit-output"])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // With --limit-output, format is: "package|version"
                let parts: Vec<&str> = line.split('|').collect();
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
    fn test_chocolatey_manager_with_progress() {
        let messages = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let messages_clone = messages.clone();

        let manager = ChocolateyManager::with_progress(move |msg| {
            messages_clone.lock().unwrap().push(msg.to_string());
        });

        manager.report_progress("test message");

        let captured = messages.lock().unwrap();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0], "test message");
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

    #[test]
    fn test_build_silent_args() {
        let manager = ChocolateyManager::new();
        let base_args = vec!["install", "imagemagick"];
        let args = manager.build_silent_args(base_args);

        assert!(args.contains(&"-y"));
        assert!(args.contains(&"--no-progress"));
        assert!(args.contains(&"--limit-output"));
    }

    #[test]
    fn test_chocolatey_priority() {
        let manager = ChocolateyManager::new();
        assert_eq!(manager.priority(), 80);
    }
}
