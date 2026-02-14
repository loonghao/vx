//! bun package installer
//!
//! Installs packages using bun (fast all-in-one JavaScript runtime).

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// bun package installer
#[derive(Debug, Clone, Default)]
pub struct BunInstaller {
    /// Path to bun executable (auto-detected if None)
    bun_path: Option<PathBuf>,
}

impl BunInstaller {
    /// Create a new bun installer
    pub fn new() -> Self {
        Self { bun_path: None }
    }

    /// Create a new bun installer with a specific bun path
    pub fn with_bun_path(bun_path: PathBuf) -> Self {
        Self {
            bun_path: Some(bun_path),
        }
    }

    /// Get the bun executable path
    fn get_bun(&self) -> Result<String> {
        if let Some(ref path) = self.bun_path {
            return Ok(path.display().to_string());
        }

        if which::which("bun").is_ok() {
            return Ok("bun".to_string());
        }

        bail!("bun not found in PATH. Please install bun first (https://bun.sh).")
    }
}

#[async_trait]
impl EcosystemInstaller for BunInstaller {
    fn ecosystem(&self) -> &'static str {
        "bun"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        let bun = self.get_bun()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        // Build package spec (package@version)
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}@{}", package, version)
        };

        // Use bun install with --global and custom prefix
        let install_dir_str = install_dir.to_string_lossy().to_string();
        let mut args = vec!["install", "--global", "--global-dir", &install_dir_str];

        // Force reinstall if requested
        if options.force {
            args.push("--force");
        }

        // Add extra arguments
        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);

        // Add package spec last
        args.push(&package_spec);

        // Build environment
        let env = self.build_install_env(install_dir);

        // Run bun install
        let output = run_command(&bun, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("bun install failed: {}", stderr);
        }

        // Detect executables
        let bin_dir = self.get_bin_dir(install_dir);
        let executables = self.detect_executables(&bin_dir)?;

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "bun".to_string(),
            install_dir.to_path_buf(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &Path) -> Result<Vec<String>> {
        detect_executables_in_dir(bin_dir)
    }

    fn build_install_env(&self, install_dir: &Path) -> InstallEnv {
        InstallEnv::new()
            .var("BUN_INSTALL_GLOBAL_DIR", install_dir.display().to_string())
            .var(
                "BUN_INSTALL_BIN",
                self.get_bin_dir(install_dir).display().to_string(),
            )
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        // bun puts binaries in global-dir/bin
        install_dir.join("bin")
    }

    fn is_available(&self) -> bool {
        self.get_bun().is_ok()
    }
}
