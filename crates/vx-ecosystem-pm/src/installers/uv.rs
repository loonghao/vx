//! uv package installer
//!
//! Installs Python packages using uv (fast Python package manager).
//! uv is significantly faster than pip and handles virtual environments automatically.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// uv package installer
///
/// Uses `uv tool install` for global tool installation with isolation.
#[derive(Debug, Clone, Default)]
pub struct UvInstaller {
    /// Path to uv executable (auto-detected if None)
    uv_path: Option<PathBuf>,
}

impl UvInstaller {
    /// Create a new uv installer
    pub fn new() -> Self {
        Self { uv_path: None }
    }

    /// Create a new uv installer with a specific uv path
    pub fn with_uv_path(uv_path: PathBuf) -> Self {
        Self {
            uv_path: Some(uv_path),
        }
    }

    /// Get the uv executable path
    fn get_uv(&self) -> Result<String> {
        if let Some(ref path) = self.uv_path {
            return Ok(path.display().to_string());
        }

        if which::which("uv").is_ok() {
            return Ok("uv".to_string());
        }

        bail!("uv not found in PATH. Please install uv first (https://docs.astral.sh/uv/).")
    }
}

#[async_trait]
impl EcosystemInstaller for UvInstaller {
    fn ecosystem(&self) -> &'static str {
        "uv"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        let uv = self.get_uv()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        // Build package spec
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}=={}", package, version)
        };

        // Use uv tool install for CLI tools (installs to isolated environment)
        // --tool-dir specifies where to install the tool
        let install_dir_str = install_dir.to_string_lossy().to_string();
        let mut args = vec!["tool", "install", "--tool-dir", &install_dir_str];

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

        // Run uv tool install
        let output = run_command(&uv, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("uv tool install failed: {}", stderr);
        }

        // Detect executables
        let bin_dir = self.get_bin_dir(install_dir);
        let mut executables = self.detect_executables(&bin_dir)?;

        // Filter out python, pip scripts
        executables.retain(|e| !e.starts_with("python") && !e.starts_with("pip"));

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "uv".to_string(),
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
            .var("UV_TOOL_DIR", install_dir.display().to_string())
            .var("UV_NO_PROGRESS", "1")
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        // uv tool puts binaries in tool-dir/bin
        if cfg!(windows) {
            install_dir.join("Scripts")
        } else {
            install_dir.join("bin")
        }
    }

    fn is_available(&self) -> bool {
        self.get_uv().is_ok()
    }
}
