//! uvx package installer
//!
//! Executes Python tools via `uvx` (uv tool run) in isolated, ephemeral environments.
//!
//! Unlike `uv tool install`, `uvx` runs each tool in a temporary virtual environment
//! that is cached by uv. This means:
//! - Each package version gets its own isolated Python environment
//! - No global state pollution
//! - Version pinning works correctly per project
//!
//! ## How it works
//!
//! `vx meson@1.5.0` → `vx uvx:meson@1.5.0` → `uvx meson==1.5.0 [args...]`
//!
//! The "install" step creates a thin shim script that delegates to `uvx`.
//! The actual Python environment is managed by uv's tool cache.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// uvx package installer
///
/// Installs Python CLI tools via `uvx` (uv tool run).
/// Each tool version runs in its own isolated, uv-managed Python environment.
#[derive(Debug, Clone, Default)]
pub struct UvxInstaller {
    /// Path to uv executable (auto-detected if None)
    uv_path: Option<PathBuf>,
}

impl UvxInstaller {
    /// Create a new uvx installer
    pub fn new() -> Self {
        Self { uv_path: None }
    }

    /// Create a new uvx installer with a specific uv path
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

    /// Build the shim script content that delegates to uvx
    ///
    /// The shim calls `uvx <package>==<version> [args...]` so that each
    /// invocation uses the correct isolated Python environment.
    fn build_shim_content(&self, package: &str, version: &str) -> String {
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}=={}", package, version)
        };

        if cfg!(windows) {
            format!("@echo off\r\nuvx {} %*\r\n", package_spec)
        } else {
            format!("#!/bin/sh\nexec uvx {} \"$@\"\n", package_spec)
        }
    }
}

#[async_trait]
impl EcosystemInstaller for UvxInstaller {
    fn ecosystem(&self) -> &'static str {
        "uvx"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        // Verify uv is available
        let _uv = self.get_uv()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        let bin_dir = self.get_bin_dir(install_dir);
        std::fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Failed to create bin directory: {}", bin_dir.display()))?;

        // Pre-warm the uvx cache for this package version so the first run is fast.
        // `uvx --no-project <package>==<version> --version` is a lightweight way to
        // trigger cache population without actually running the tool's main logic.
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}=={}", package, version)
        };

        // Use `uv tool install` to pre-install into the tool cache
        // This makes subsequent `uvx` calls instant (cache hit)
        let uv = self.get_uv()?;
        let mut args = vec!["tool", "install"];

        if options.force {
            args.push("--force");
        }

        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);
        args.push(&package_spec);

        let env = self.build_install_env(install_dir);
        let output = crate::utils::run_command(&uv, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Non-fatal: uvx can still run even if pre-install fails
            tracing::warn!(
                "uv tool install pre-warm failed for {}: {}",
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
            "uvx".to_string(),
            install_dir.to_path_buf(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &Path) -> Result<Vec<String>> {
        crate::utils::detect_executables_in_dir(bin_dir)
    }

    fn build_install_env(&self, _install_dir: &Path) -> InstallEnv {
        InstallEnv::new().var("UV_NO_PROGRESS", "1")
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        install_dir.join("bin")
    }

    fn is_available(&self) -> bool {
        self.get_uv().is_ok()
    }
}
