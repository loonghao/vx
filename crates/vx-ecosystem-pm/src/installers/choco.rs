//! Chocolatey package installer
//!
//! Installs Windows packages using Chocolatey package manager.
//! Packages are installed to an isolated `choco_tools` directory managed by vx,
//! rather than the default Chocolatey install location.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Chocolatey package installer
///
/// Uses `choco install` to install Windows packages to a vx-managed directory.
/// All packages are installed to `~/.vx/choco-tools/` for unified management.
///
/// ## Features
/// - Isolated installation to vx-managed directory
/// - Progress reporting via choco's verbose output
/// - Automatic executable detection
/// - Clean uninstall support
#[derive(Debug, Clone, Default)]
pub struct ChocoInstaller {
    /// Path to choco executable (auto-detected if None)
    choco_path: Option<PathBuf>,
}

impl ChocoInstaller {
    /// Create a new choco installer
    pub fn new() -> Self {
        Self { choco_path: None }
    }

    /// Create a new choco installer with a specific choco path
    pub fn with_choco_path(choco_path: PathBuf) -> Self {
        Self {
            choco_path: Some(choco_path),
        }
    }

    /// Get the choco executable path
    fn get_choco(&self) -> Result<String> {
        if let Some(ref path) = self.choco_path {
            return Ok(path.display().to_string());
        }

        // Check if choco is available in PATH
        if which::which("choco").is_ok() {
            return Ok("choco".to_string());
        }

        // Check common installation locations on Windows
        #[cfg(windows)]
        {
            let common_paths = [
                r"C:\ProgramData\chocolatey\bin\choco.exe",
                r"C:\ProgramData\chocolatey\choco.exe",
            ];
            for path in &common_paths {
                if Path::new(path).exists() {
                    return Ok(path.to_string());
                }
            }
        }

        bail!(
            "Chocolatey not found. Install it with:\n\
             \n\
               vx install choco\n\
             \n\
             Or visit: https://chocolatey.org/install"
        )
    }
}

#[async_trait]
impl EcosystemInstaller for ChocoInstaller {
    fn ecosystem(&self) -> &'static str {
        "choco"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        let choco = self.get_choco()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        // Build choco install command
        // --install-directory redirects installation to our managed directory
        let install_dir_str = install_dir.to_string_lossy().to_string();
        let mut args = vec![
            "install",
            package,
            "--yes",         // Non-interactive
            "--no-progress", // Cleaner output for parsing
            "--install-directory",
            &install_dir_str,
        ];

        // Add version constraint if specified
        let version_flag;
        if version != "latest" {
            version_flag = format!("--version={}", version);
            args.push(&version_flag);
        }

        // Force reinstall if requested
        if options.force {
            args.push("--force");
        }

        // Add extra arguments
        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);

        // Build environment
        let env = self.build_install_env(install_dir);

        // Run choco install
        let output = run_command(&choco, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            bail!(
                "choco install failed:\n{}\n{}",
                stdout.trim(),
                stderr.trim()
            );
        }

        // Detect executables in the install directory
        let bin_dir = self.get_bin_dir(install_dir);
        let executables = self.detect_executables(&bin_dir)?;

        // Parse actual installed version from choco output
        let actual_version = Self::parse_installed_version(&output.stdout, package)
            .unwrap_or_else(|| version.to_string());

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            actual_version,
            "choco".to_string(),
            install_dir.to_path_buf(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &Path) -> Result<Vec<String>> {
        // Choco installs executables in multiple possible locations
        // Check the main dir, tools/ subdirectory, and bin/ subdirectory
        let mut all_executables = Vec::new();

        // Check the bin_dir itself
        if let Ok(mut exes) = detect_executables_in_dir(bin_dir) {
            all_executables.append(&mut exes);
        }

        // Check tools/ subdirectory (common choco package layout)
        let tools_dir = bin_dir.join("tools");
        if tools_dir.exists()
            && let Ok(mut exes) = detect_executables_in_dir(&tools_dir)
        {
            all_executables.append(&mut exes);
        }

        // Deduplicate
        all_executables.sort();
        all_executables.dedup();

        Ok(all_executables)
    }

    fn build_install_env(&self, install_dir: &Path) -> InstallEnv {
        // Set ChocolateyToolsLocation to redirect package tools to our managed dir
        InstallEnv::new()
            .var("ChocolateyToolsLocation", install_dir.display().to_string())
            .var("ChocolateyInstall", install_dir.display().to_string())
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        // Choco packages typically put executables directly in the install dir
        // or in a tools/ subdirectory
        install_dir.to_path_buf()
    }

    fn uninstall(&self, install_dir: &Path) -> Result<()> {
        // For choco packages, we try to run `choco uninstall` first
        // then clean up the directory
        if install_dir.exists() {
            // Extract package name from directory structure
            // install_dir is typically: ~/.vx/packages/choco/{package}/{version}
            if let Some(package_name) = install_dir
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                && let Ok(choco) = self.get_choco()
            {
                let env = InstallEnv::new();
                // Try choco uninstall (best effort)
                let _ = run_command(
                    &choco,
                    &["uninstall", package_name, "--yes", "--no-progress"],
                    &env,
                    false,
                );
            }

            // Clean up the directory regardless
            std::fs::remove_dir_all(install_dir)
                .with_context(|| format!("Failed to remove {}", install_dir.display()))?;
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.get_choco().is_ok()
    }
}

impl ChocoInstaller {
    /// Parse the installed version from choco output
    fn parse_installed_version(output: &[u8], _package: &str) -> Option<String> {
        let stdout = String::from_utf8_lossy(output);
        // choco output typically contains lines like:
        // "packagename v1.2.3 [Approved]"
        // or "The install of packagename was successful."
        for line in stdout.lines() {
            let line = line.trim();
            // Look for version pattern in the output
            if line.contains(" v") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    if part.starts_with('v')
                        && part.len() > 1
                        && part[1..].chars().next().is_some_and(|c| c.is_ascii_digit())
                    {
                        return Some(parts[i][1..].to_string());
                    }
                }
            }
        }
        None
    }
}
