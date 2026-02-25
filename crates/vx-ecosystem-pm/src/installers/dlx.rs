//! dlx package installer
//!
//! Runs npm packages via `pnpm dlx` in isolated, ephemeral environments.
//!
//! `pnpm dlx` is pnpm's equivalent of `npx` - it fetches a package from the
//! registry, puts it in a temporary location, and runs it without installing
//! it as a dependency.
//!
//! ## How it works
//!
//! `vx dlx:create-react-app my-app` → `pnpm dlx create-react-app my-app`
//!
//! The "install" step creates a thin shim script that delegates to `pnpm dlx`.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// dlx package installer (pnpm dlx)
///
/// Runs npm packages via `pnpm dlx` in isolated, ephemeral environments.
#[derive(Debug, Clone, Default)]
pub struct DlxInstaller {
    /// Path to pnpm executable (auto-detected if None)
    pnpm_path: Option<PathBuf>,
}

impl DlxInstaller {
    /// Create a new dlx installer
    pub fn new() -> Self {
        Self { pnpm_path: None }
    }

    /// Create a new dlx installer with a specific pnpm path
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
        if cfg!(windows) && which::which("pnpm.cmd").is_ok() {
            return Ok("pnpm.cmd".to_string());
        }

        if which::which("pnpm").is_ok() {
            return Ok("pnpm".to_string());
        }

        bail!("pnpm not found in PATH. Please install pnpm first (https://pnpm.io).")
    }

    /// Build the shim script content that delegates to `pnpm dlx`
    fn build_shim_content(&self, package: &str, version: &str) -> String {
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}@{}", package, version)
        };

        if cfg!(windows) {
            format!("@echo off\r\npnpm dlx {} %*\r\n", package_spec)
        } else {
            format!("#!/bin/sh\nexec pnpm dlx {} \"$@\"\n", package_spec)
        }
    }
}

#[async_trait]
impl EcosystemInstaller for DlxInstaller {
    fn ecosystem(&self) -> &'static str {
        "dlx"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        // Verify pnpm is available
        let _pnpm = self.get_pnpm()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        let bin_dir = self.get_bin_dir(install_dir);
        std::fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Failed to create bin directory: {}", bin_dir.display()))?;

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

        let _ = options; // dlx is ephemeral, no pre-install needed

        let executables = vec![package.to_string()];

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "dlx".to_string(),
            install_dir.to_path_buf(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &Path) -> Result<Vec<String>> {
        crate::utils::detect_executables_in_dir(bin_dir)
    }

    fn build_install_env(&self, _install_dir: &Path) -> InstallEnv {
        InstallEnv::new()
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        install_dir.join("bin")
    }

    fn is_available(&self) -> bool {
        self.get_pnpm().is_ok()
    }
}
