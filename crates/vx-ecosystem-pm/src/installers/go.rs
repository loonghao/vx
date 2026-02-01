//! go package installer
//!
//! Installs Go packages using `go install` with GOBIN redirection.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

/// go package installer
#[derive(Debug, Clone, Default)]
pub struct GoInstaller {
    /// Path to go executable (auto-detected if None)
    go_path: Option<PathBuf>,
}

impl GoInstaller {
    /// Create a new go installer
    pub fn new() -> Self {
        Self { go_path: None }
    }

    /// Create a new go installer with a specific go path
    pub fn with_go_path(go_path: PathBuf) -> Self {
        Self {
            go_path: Some(go_path),
        }
    }

    /// Get the go executable path
    fn get_go(&self) -> Result<String> {
        if let Some(ref path) = self.go_path {
            return Ok(path.display().to_string());
        }

        if which::which("go").is_ok() {
            return Ok("go".to_string());
        }

        bail!("go not found in PATH. Please install Go first.")
    }
}

#[async_trait]
impl EcosystemInstaller for GoInstaller {
    fn ecosystem(&self) -> &'static str {
        "go"
    }

    async fn install(
        &self,
        install_dir: &PathBuf,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        let go = self.get_go()?;
        let bin_dir = self.get_bin_dir(install_dir);

        // Create directories
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;
        std::fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Failed to create directory: {}", bin_dir.display()))?;

        // Build package spec for go install
        // Format: package@version or package@latest
        let package_spec = if version == "latest" {
            format!("{}@latest", package)
        } else {
            format!("{}@v{}", package, version.trim_start_matches('v'))
        };

        // Build arguments
        let mut args = vec!["install"];

        // Add extra arguments
        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);

        // Add package spec last
        args.push(&package_spec);

        // Build environment with GOBIN
        let env = self.build_install_env(install_dir);

        // Run go install
        let output = run_command(&go, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("go install failed: {}", stderr);
        }

        // Detect executables
        let executables = self.detect_executables(&bin_dir)?;

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "go".to_string(),
            install_dir.clone(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &PathBuf) -> Result<Vec<String>> {
        detect_executables_in_dir(bin_dir)
    }

    fn build_install_env(&self, install_dir: &PathBuf) -> InstallEnv {
        let bin_dir = self.get_bin_dir(install_dir);

        InstallEnv::new()
            // Redirect go install binaries to our isolated directory
            .var("GOBIN", bin_dir.display().to_string())
    }

    fn get_bin_dir(&self, install_dir: &PathBuf) -> PathBuf {
        // go install puts binaries in GOBIN
        install_dir.join("bin")
    }

    fn is_available(&self) -> bool {
        self.get_go().is_ok()
    }
}

