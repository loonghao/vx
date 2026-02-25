//! pipx package installer
//!
//! Runs Python CLI tools via `pipx run` in isolated, ephemeral environments.
//!
//! `pipx run` is similar to `uvx` but uses pip under the hood.
//! Each tool runs in its own temporary virtual environment.
//!
//! ## How it works
//!
//! `vx pipx:cowsay` → `pipx run cowsay [args...]`
//!
//! The "install" step creates a thin shim script that delegates to `pipx run`.
//! The actual Python environment is managed by pipx's cache.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// pipx package installer
///
/// Runs Python CLI tools via `pipx run` in isolated, ephemeral environments.
#[derive(Debug, Clone, Default)]
pub struct PipxInstaller {
    /// Path to pipx executable (auto-detected if None)
    pipx_path: Option<PathBuf>,
}

impl PipxInstaller {
    /// Create a new pipx installer
    pub fn new() -> Self {
        Self { pipx_path: None }
    }

    /// Create a new pipx installer with a specific pipx path
    pub fn with_pipx_path(pipx_path: PathBuf) -> Self {
        Self {
            pipx_path: Some(pipx_path),
        }
    }

    /// Get the pipx executable path
    fn get_pipx(&self) -> Result<String> {
        if let Some(ref path) = self.pipx_path {
            return Ok(path.display().to_string());
        }

        if which::which("pipx").is_ok() {
            return Ok("pipx".to_string());
        }

        bail!(
            "pipx not found in PATH. Please install pipx first (https://pipx.pypa.io).\nAlternatively, use 'uvx' ecosystem which provides similar functionality."
        );
    }

    /// Build the shim script content that delegates to `pipx run`
    fn build_shim_content(&self, package: &str, version: &str) -> String {
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}=={}", package, version)
        };

        if cfg!(windows) {
            format!("@echo off\r\npipx run {} %*\r\n", package_spec)
        } else {
            format!("#!/bin/sh\nexec pipx run {} \"$@\"\n", package_spec)
        }
    }
}

#[async_trait]
impl EcosystemInstaller for PipxInstaller {
    fn ecosystem(&self) -> &'static str {
        "pipx"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        // Verify pipx is available
        let _pipx = self.get_pipx()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        let bin_dir = self.get_bin_dir(install_dir);
        std::fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Failed to create bin directory: {}", bin_dir.display()))?;

        // Pre-warm the pipx cache by running `pipx install` into a temp venv
        // This makes subsequent `pipx run` calls faster (cache hit)
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}=={}", package, version)
        };

        let pipx = self.get_pipx()?;
        let mut args = vec!["install"];

        if options.force {
            args.push("--force");
        }

        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);
        args.push(&package_spec);

        let env = self.build_install_env(install_dir);
        let output = crate::utils::run_command(&pipx, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Non-fatal: pipx run can still work even if pre-install fails
            tracing::warn!(
                "pipx install pre-warm failed for {}: {}",
                package_spec,
                stderr
            );
        }

        // Create a thin shim script in the bin directory
        let shim_name = if cfg!(windows) {
            format!("{}.cmd", package)
        } else {
            package.to_string()
        };
        let shim_path = bin_dir.join(&shim_name);
        let shim_content = self.build_shim_content(package, version);

        std::fs::write(&shim_path, &shim_content)
            .with_context(|| format!("Failed to write shim: {}", shim_path.display()))?;

        // Make shim executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&shim_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&shim_path, perms)?;
        }

        let executables = vec![package.to_string()];

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "pipx".to_string(),
            install_dir.to_path_buf(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &Path) -> Result<Vec<String>> {
        crate::utils::detect_executables_in_dir(bin_dir)
    }

    fn build_install_env(&self, _install_dir: &Path) -> InstallEnv {
        InstallEnv::new().var("PIPX_DEFAULT_PYTHON", "python3")
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        install_dir.join("bin")
    }

    fn is_available(&self) -> bool {
        self.get_pipx().is_ok()
    }
}
