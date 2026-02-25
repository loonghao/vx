//! deno package installer
//!
//! Runs npm/JSR packages via `deno run` in isolated, ephemeral environments.
//!
//! Deno has built-in support for running npm packages and JSR packages
//! without a separate install step. It also supports running remote scripts.
//!
//! ## How it works
//!
//! `vx deno:cowsay` → `deno run npm:cowsay [args...]`
//! `vx deno:@std/cli` → `deno run jsr:@std/cli [args...]`
//!
//! The "install" step creates a thin shim script that delegates to `deno run`.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// deno package installer
///
/// Runs npm/JSR packages via `deno run` in isolated, ephemeral environments.
#[derive(Debug, Clone, Default)]
pub struct DenoInstaller {
    /// Path to deno executable (auto-detected if None)
    deno_path: Option<PathBuf>,
}

impl DenoInstaller {
    /// Create a new deno installer
    pub fn new() -> Self {
        Self { deno_path: None }
    }

    /// Create a new deno installer with a specific deno path
    pub fn with_deno_path(deno_path: PathBuf) -> Self {
        Self {
            deno_path: Some(deno_path),
        }
    }

    /// Get the deno executable path
    fn get_deno(&self) -> Result<String> {
        if let Some(ref path) = self.deno_path {
            return Ok(path.display().to_string());
        }

        if which::which("deno").is_ok() {
            return Ok("deno".to_string());
        }

        bail!(
            "deno not found in PATH. Please install deno first (https://deno.land/#installation)."
        )
    }

    /// Build the package specifier for deno run
    ///
    /// Deno supports:
    /// - `npm:package@version` for npm packages
    /// - `jsr:@scope/package@version` for JSR packages
    /// - Direct URLs for remote scripts
    fn build_package_spec(package: &str, version: &str) -> String {
        let version_suffix = if version == "latest" {
            String::new()
        } else {
            format!("@{}", version)
        };

        if package.starts_with("jsr:") || package.starts_with("npm:") || package.starts_with("http")
        {
            // Already has a specifier prefix
            package.to_string()
        } else if package.starts_with('@') {
            // Scoped package - could be JSR or npm, default to npm
            format!("npm:{}{}", package, version_suffix)
        } else {
            // Regular npm package
            format!("npm:{}{}", package, version_suffix)
        }
    }

    /// Build the shim script content that delegates to `deno run`
    fn build_shim_content(&self, package: &str, version: &str) -> String {
        let package_spec = Self::build_package_spec(package, version);

        // Use the full deno path if available, otherwise just "deno"
        let deno_cmd = self.get_deno().unwrap_or_else(|_| "deno".to_string());

        if cfg!(windows) {
            format!(
                "@echo off\r\n\"{}\" run --allow-all {} %*\r\n",
                deno_cmd, package_spec
            )
        } else {
            format!(
                "#!/bin/sh\nexec \"{}\" run --allow-all {} \"$@\"\n",
                deno_cmd, package_spec
            )
        }
    }
}

#[async_trait]
impl EcosystemInstaller for DenoInstaller {
    fn ecosystem(&self) -> &'static str {
        "deno"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        // Verify deno is available
        let _deno = self.get_deno()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        let bin_dir = self.get_bin_dir(install_dir);
        std::fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Failed to create bin directory: {}", bin_dir.display()))?;

        // Pre-warm deno cache for this package
        let package_spec = Self::build_package_spec(package, version);
        let deno = self.get_deno()?;

        // Use `deno install` to create a global executable
        let install_dir_str = install_dir.to_string_lossy().to_string();
        let mut args = vec!["install", "--root", &install_dir_str];

        if options.force {
            args.push("--force");
        }

        // Allow all permissions for installed tools
        args.push("--allow-all");

        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);
        args.push(&package_spec);

        let env = self.build_install_env(install_dir);
        let output = crate::utils::run_command(&deno, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Non-fatal: create shim as fallback
            tracing::warn!(
                "deno install pre-warm failed for {}: {}",
                package_spec,
                stderr
            );
        }

        // Create a thin shim script as fallback
        let shim_name = if cfg!(windows) {
            format!("{}.cmd", package)
        } else {
            package.to_string()
        };
        let shim_path = bin_dir.join(&shim_name);

        // Only create shim if deno install didn't create the executable
        if !shim_path.exists() {
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
        }

        let executables = vec![package.to_string()];

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "deno".to_string(),
            install_dir.to_path_buf(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &Path) -> Result<Vec<String>> {
        crate::utils::detect_executables_in_dir(bin_dir)
    }

    fn build_install_env(&self, _install_dir: &Path) -> InstallEnv {
        InstallEnv::new().var("DENO_NO_UPDATE_CHECK", "1")
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        // deno install --root puts executables in <root>/bin
        install_dir.join("bin")
    }

    fn is_available(&self) -> bool {
        self.get_deno().is_ok()
    }
}
