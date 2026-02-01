//! gem package installer
//!
//! Installs Ruby gems using GEM_HOME redirection.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// gem package installer
#[derive(Debug, Clone, Default)]
pub struct GemInstaller {
    /// Path to gem executable (auto-detected if None)
    gem_path: Option<PathBuf>,
}

impl GemInstaller {
    /// Create a new gem installer
    pub fn new() -> Self {
        Self { gem_path: None }
    }

    /// Create a new gem installer with a specific gem path
    pub fn with_gem_path(gem_path: PathBuf) -> Self {
        Self {
            gem_path: Some(gem_path),
        }
    }

    /// Get the gem executable path
    fn get_gem(&self) -> Result<String> {
        if let Some(ref path) = self.gem_path {
            return Ok(path.display().to_string());
        }

        if which::which("gem").is_ok() {
            return Ok("gem".to_string());
        }

        bail!("gem not found in PATH. Please install Ruby first.")
    }
}

#[async_trait]
impl EcosystemInstaller for GemInstaller {
    fn ecosystem(&self) -> &'static str {
        "gem"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        let gem = self.get_gem()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        // Build arguments
        let mut args = vec!["install", package];

        // Add version if not latest
        let version_arg;
        if version != "latest" {
            version_arg = version.to_string();
            args.push("--version");
            args.push(&version_arg);
        }

        // No documentation to speed up installation
        args.push("--no-document");

        // Force reinstall if requested
        if options.force {
            args.push("--force");
        }

        // Add extra arguments
        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);

        // Build environment with GEM_HOME
        let env = self.build_install_env(install_dir);

        // Run gem install
        let output = run_command(&gem, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("gem install failed: {}", stderr);
        }

        // Detect executables
        let bin_dir = self.get_bin_dir(install_dir);
        let executables = self.detect_executables(&bin_dir)?;

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "gem".to_string(),
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
            // Redirect gem install to our isolated directory
            .var("GEM_HOME", install_dir.display().to_string())
            .var("GEM_PATH", install_dir.display().to_string())
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        // gem puts binaries in GEM_HOME/bin
        install_dir.join("bin")
    }

    fn is_available(&self) -> bool {
        self.get_gem().is_ok()
    }
}
