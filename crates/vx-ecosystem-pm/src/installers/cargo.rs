//! cargo package installer
//!
//! Installs Rust packages using `cargo install` with CARGO_INSTALL_ROOT redirection.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

/// cargo package installer
#[derive(Debug, Clone, Default)]
pub struct CargoInstaller {
    /// Path to cargo executable (auto-detected if None)
    cargo_path: Option<PathBuf>,
}

impl CargoInstaller {
    /// Create a new cargo installer
    pub fn new() -> Self {
        Self { cargo_path: None }
    }

    /// Create a new cargo installer with a specific cargo path
    pub fn with_cargo_path(cargo_path: PathBuf) -> Self {
        Self {
            cargo_path: Some(cargo_path),
        }
    }

    /// Get the cargo executable path
    fn get_cargo(&self) -> Result<String> {
        if let Some(ref path) = self.cargo_path {
            return Ok(path.display().to_string());
        }

        if which::which("cargo").is_ok() {
            return Ok("cargo".to_string());
        }

        bail!("cargo not found in PATH. Please install Rust first.")
    }
}

#[async_trait]
impl EcosystemInstaller for CargoInstaller {
    fn ecosystem(&self) -> &'static str {
        "cargo"
    }

    async fn install(
        &self,
        install_dir: &PathBuf,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        let cargo = self.get_cargo()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        // Build arguments
        let mut args = vec!["install"];

        // Add version if not latest
        let version_arg;
        if version != "latest" {
            version_arg = version.to_string();
            args.push("--version");
            args.push(&version_arg);
        }

        // Force reinstall if requested
        if options.force {
            args.push("--force");
        }

        // Add extra arguments
        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);

        // Add package name last
        args.push(package);

        // Build environment with CARGO_INSTALL_ROOT
        let env = self.build_install_env(install_dir);

        // Run cargo install
        let output = run_command(&cargo, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("cargo install failed: {}", stderr);
        }

        // Detect executables
        let bin_dir = self.get_bin_dir(install_dir);
        let executables = self.detect_executables(&bin_dir)?;

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "cargo".to_string(),
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
            // Redirect cargo install to our isolated directory
            .var("CARGO_INSTALL_ROOT", install_dir.display().to_string())
    }

    fn get_bin_dir(&self, install_dir: &PathBuf) -> PathBuf {
        // cargo install puts binaries in CARGO_INSTALL_ROOT/bin
        install_dir.join("bin")
    }

    fn is_available(&self) -> bool {
        self.get_cargo().is_ok()
    }
}

