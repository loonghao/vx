//! pnpm package installer
//!
//! Installs packages using pnpm (fast, disk space efficient package manager).

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

/// pnpm package installer
#[derive(Debug, Clone, Default)]
pub struct PnpmInstaller {
    /// Path to pnpm executable (auto-detected if None)
    pnpm_path: Option<PathBuf>,
}

impl PnpmInstaller {
    /// Create a new pnpm installer
    pub fn new() -> Self {
        Self { pnpm_path: None }
    }

    /// Create a new pnpm installer with a specific pnpm path
    pub fn with_pnpm_path(pnpm_path: PathBuf) -> Self {
        Self {
            pnpm_path: Some(pnpm_path),
        }
    }

    /// Get the pnpm executable path
    fn get_pnpm(&self) -> Result<String> {
        if let Some(ref path) = self.pnpm_path {
            return Ok(path.display().to_string());
        }

        // On Windows, prefer pnpm.cmd
        if cfg!(windows) {
            if which::which("pnpm.cmd").is_ok() {
                return Ok("pnpm.cmd".to_string());
            }
        }

        if which::which("pnpm").is_ok() {
            return Ok("pnpm".to_string());
        }

        bail!("pnpm not found in PATH. Please install pnpm first (https://pnpm.io).")
    }
}

#[async_trait]
impl EcosystemInstaller for PnpmInstaller {
    fn ecosystem(&self) -> &'static str {
        "pnpm"
    }

    async fn install(
        &self,
        install_dir: &PathBuf,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        let pnpm = self.get_pnpm()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        // Build package spec (package@version)
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}@{}", package, version)
        };

        // Use pnpm add --global with custom global-dir
        let install_dir_str = install_dir.to_string_lossy().to_string();
        let mut args = vec!["add", "--global", "--global-dir", &install_dir_str];

        // Add extra arguments
        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);

        // Add package spec last
        args.push(&package_spec);

        // Build environment
        let env = self.build_install_env(install_dir);

        // Run pnpm add --global
        let output = run_command(&pnpm, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("pnpm add --global failed: {}", stderr);
        }

        // Detect executables
        let bin_dir = self.get_bin_dir(install_dir);
        let executables = self.detect_executables(&bin_dir)?;

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "pnpm".to_string(),
            install_dir.clone(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &PathBuf) -> Result<Vec<String>> {
        detect_executables_in_dir(bin_dir)
    }

    fn build_install_env(&self, install_dir: &PathBuf) -> InstallEnv {
        InstallEnv::new()
            .var("PNPM_HOME", install_dir.display().to_string())
            .var("npm_config_global_dir", install_dir.display().to_string())
    }

    fn get_bin_dir(&self, install_dir: &PathBuf) -> PathBuf {
        // pnpm global puts binaries in global-dir (directly, not in bin subdirectory)
        // But with --global-bin-dir we can control it
        install_dir.clone()
    }

    fn is_available(&self) -> bool {
        self.get_pnpm().is_ok()
    }
}

