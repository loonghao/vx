//! jbang package installer
//!
//! Runs Java tools via `jbang` - a scripting solution for Java.
//!
//! JBang allows running Java programs without a build system, downloading
//! dependencies automatically. It supports running from Maven Central,
//! GitHub, and other sources.
//!
//! ## How it works
//!
//! `vx jbang:picocli@info.picocli` → `jbang run info.picocli:picocli [args...]`
//!
//! The "install" step creates a thin shim script that delegates to `jbang`.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// jbang package installer
///
/// Runs Java tools via `jbang` in isolated, cached environments.
#[derive(Debug, Clone, Default)]
pub struct JBangInstaller {
    /// Path to jbang executable (auto-detected if None)
    jbang_path: Option<PathBuf>,
}

impl JBangInstaller {
    /// Create a new jbang installer
    pub fn new() -> Self {
        Self { jbang_path: None }
    }

    /// Create a new jbang installer with a specific jbang path
    pub fn with_jbang_path(jbang_path: PathBuf) -> Self {
        Self {
            jbang_path: Some(jbang_path),
        }
    }

    /// Get the jbang executable path
    fn get_jbang(&self) -> Result<String> {
        if let Some(ref path) = self.jbang_path {
            return Ok(path.display().to_string());
        }

        if which::which("jbang").is_ok() {
            return Ok("jbang".to_string());
        }

        bail!(
            "jbang not found in PATH. Please install jbang first (https://www.jbang.dev/download/)."
        )
    }

    /// Build the shim script content that delegates to `jbang`
    fn build_shim_content(&self, package: &str, version: &str) -> String {
        // jbang supports GAV (groupId:artifactId:version) format
        // For simple package names, use as-is
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            // If package already contains ':', it's a GAV coordinate
            if package.contains(':') {
                format!("{}:{}", package, version)
            } else {
                format!("{}@{}", package, version)
            }
        };

        if cfg!(windows) {
            format!("@echo off\r\njbang run {} %*\r\n", package_spec)
        } else {
            format!("#!/bin/sh\nexec jbang run {} \"$@\"\n", package_spec)
        }
    }
}

#[async_trait]
impl EcosystemInstaller for JBangInstaller {
    fn ecosystem(&self) -> &'static str {
        "jbang"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        // Verify jbang is available
        let _jbang = self.get_jbang()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        let bin_dir = self.get_bin_dir(install_dir);
        std::fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Failed to create bin directory: {}", bin_dir.display()))?;

        // Pre-warm jbang cache
        let package_spec = if version == "latest" {
            package.to_string()
        } else if package.contains(':') {
            format!("{}:{}", package, version)
        } else {
            format!("{}@{}", package, version)
        };

        let jbang = self.get_jbang()?;
        // Use `jbang app install` to make the tool available
        let mut args = vec!["app", "install"];

        if options.force {
            args.push("--force");
        }

        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);
        args.push(&package_spec);

        let env = self.build_install_env(install_dir);
        let output = crate::utils::run_command(&jbang, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Non-fatal: jbang run can still work even if app install fails
            tracing::warn!(
                "jbang app install pre-warm failed for {}: {}",
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
            "jbang".to_string(),
            install_dir.to_path_buf(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &Path) -> Result<Vec<String>> {
        crate::utils::detect_executables_in_dir(bin_dir)
    }

    fn build_install_env(&self, _install_dir: &Path) -> InstallEnv {
        InstallEnv::new().var("JBANG_NO_VERSION_CHECK", "true")
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        install_dir.join("bin")
    }

    fn is_available(&self) -> bool {
        self.get_jbang().is_ok()
    }
}
