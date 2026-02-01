//! yarn package installer
//!
//! Installs packages using yarn (fast, reliable, and secure dependency management).

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// yarn package installer
#[derive(Debug, Clone, Default)]
pub struct YarnInstaller {
    /// Path to yarn executable (auto-detected if None)
    yarn_path: Option<PathBuf>,
}

impl YarnInstaller {
    /// Create a new yarn installer
    pub fn new() -> Self {
        Self { yarn_path: None }
    }

    /// Create a new yarn installer with a specific yarn path
    pub fn with_yarn_path(yarn_path: PathBuf) -> Self {
        Self {
            yarn_path: Some(yarn_path),
        }
    }

    /// Get the yarn executable path
    fn get_yarn(&self) -> Result<String> {
        if let Some(ref path) = self.yarn_path {
            return Ok(path.display().to_string());
        }

        // On Windows, prefer yarn.cmd
        if cfg!(windows) && which::which("yarn.cmd").is_ok() {
            return Ok("yarn.cmd".to_string());
        }

        if which::which("yarn").is_ok() {
            return Ok("yarn".to_string());
        }

        bail!("yarn not found in PATH. Please install yarn first (https://yarnpkg.com).")
    }
}

#[async_trait]
impl EcosystemInstaller for YarnInstaller {
    fn ecosystem(&self) -> &'static str {
        "yarn"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        let yarn = self.get_yarn()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        // Build package spec (package@version)
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}@{}", package, version)
        };

        // Use yarn global add with custom prefix
        let install_dir_str = install_dir.to_string_lossy().to_string();
        let mut args = vec!["global", "add", "--prefix", &install_dir_str];

        // Add extra arguments
        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);

        // Add package spec last
        args.push(&package_spec);

        // Build environment
        let env = self.build_install_env(install_dir);

        // Run yarn global add
        let output = run_command(&yarn, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("yarn global add failed: {}", stderr);
        }

        // Detect executables
        let bin_dir = self.get_bin_dir(install_dir);
        let executables = self.detect_executables(&bin_dir)?;

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "yarn".to_string(),
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
            .var("YARN_GLOBAL_FOLDER", install_dir.display().to_string())
            .var("YARN_SILENT", "1")
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        // yarn global puts binaries in prefix/bin
        install_dir.join("bin")
    }

    fn is_available(&self) -> bool {
        self.get_yarn().is_ok()
    }
}
