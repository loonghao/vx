//! npm package installer
//!
//! Installs npm packages to isolated directories using `npm install --prefix`.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

/// npm package installer
#[derive(Debug, Clone, Default)]
pub struct NpmInstaller {
    /// Path to npm executable (auto-detected if None)
    npm_path: Option<PathBuf>,
}

impl NpmInstaller {
    /// Create a new npm installer
    pub fn new() -> Self {
        Self { npm_path: None }
    }

    /// Create a new npm installer with a specific npm path
    pub fn with_npm_path(npm_path: PathBuf) -> Self {
        Self {
            npm_path: Some(npm_path),
        }
    }

    /// Get the npm executable path
    fn get_npm(&self) -> Result<String> {
        if let Some(ref path) = self.npm_path {
            return Ok(path.display().to_string());
        }

        // On Windows, prefer npm.cmd
        if cfg!(windows) {
            if which::which("npm.cmd").is_ok() {
                return Ok("npm.cmd".to_string());
            }
        }

        if which::which("npm").is_ok() {
            return Ok("npm".to_string());
        }

        bail!("npm not found in PATH. Please install Node.js first or specify npm path.")
    }
}

#[async_trait]
impl EcosystemInstaller for NpmInstaller {
    fn ecosystem(&self) -> &'static str {
        "npm"
    }

    async fn install(
        &self,
        install_dir: &PathBuf,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        let npm = self.get_npm()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        // Build package spec (package@version)
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}@{}", package, version)
        };

        // Build arguments
        let install_dir_str = install_dir.to_string_lossy().to_string();
        let mut args = vec!["install", "--prefix", &install_dir_str, "--global"];

        // Add extra arguments
        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);

        // Add package spec last
        args.push(&package_spec);

        // Build environment
        let env = self.build_install_env(install_dir);

        // Run npm install
        let output = run_command(&npm, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("npm install failed: {}", stderr);
        }

        // Detect executables
        let bin_dir = self.get_bin_dir(install_dir);
        let executables = self.detect_executables(&bin_dir)?;

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "npm".to_string(),
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
            .var("NPM_CONFIG_PREFIX", install_dir.display().to_string())
            .var("NO_UPDATE_NOTIFIER", "1")
    }

    fn get_bin_dir(&self, install_dir: &PathBuf) -> PathBuf {
        if cfg!(windows) {
            // On Windows, npm installs binaries directly in the prefix directory
            install_dir.clone()
        } else {
            // On Unix, npm installs binaries in prefix/bin
            install_dir.join("bin")
        }
    }

    fn is_available(&self) -> bool {
        self.get_npm().is_ok()
    }
}

