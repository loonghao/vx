//! pip package installer
//!
//! Installs pip packages to isolated virtual environments.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// pip package installer
#[derive(Debug, Clone, Default)]
pub struct PipInstaller {
    /// Path to python executable (auto-detected if None)
    python_path: Option<PathBuf>,
}

impl PipInstaller {
    /// Create a new pip installer
    pub fn new() -> Self {
        Self { python_path: None }
    }

    /// Create a new pip installer with a specific python path
    pub fn with_python_path(python_path: PathBuf) -> Self {
        Self {
            python_path: Some(python_path),
        }
    }

    /// Get the python executable path
    fn get_python(&self) -> Result<String> {
        if let Some(ref path) = self.python_path {
            return Ok(path.display().to_string());
        }

        // Try common python names
        for name in &["python3", "python", "py"] {
            if which::which(name).is_ok() {
                return Ok(name.to_string());
            }
        }

        bail!("Python not found in PATH. Please install Python first.")
    }

    /// Get pip executable in venv
    fn get_venv_pip(&self, venv_dir: &Path) -> PathBuf {
        if cfg!(windows) {
            venv_dir.join("Scripts").join("pip.exe")
        } else {
            venv_dir.join("bin").join("pip")
        }
    }

    /// Create a virtual environment
    fn create_venv(&self, venv_dir: &Path, verbose: bool) -> Result<()> {
        let python = self.get_python()?;

        let venv_dir_str = venv_dir.to_string_lossy().to_string();
        let args = ["-m", "venv", &venv_dir_str];
        let env = InstallEnv::new();

        let output = run_command(&python, &args, &env, verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to create virtual environment: {}", stderr);
        }

        Ok(())
    }
}

#[async_trait]
impl EcosystemInstaller for PipInstaller {
    fn ecosystem(&self) -> &'static str {
        "pip"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        // For pip, install_dir is the venv directory
        let venv_dir = install_dir;

        // Create installation directory
        std::fs::create_dir_all(venv_dir)
            .with_context(|| format!("Failed to create directory: {}", venv_dir.display()))?;

        // Create virtual environment
        self.create_venv(venv_dir, options.verbose)?;

        // Get pip in venv
        let pip = self.get_venv_pip(venv_dir);
        if !pip.exists() {
            bail!("pip not found in virtual environment");
        }

        // Build package spec
        let package_spec = if version == "latest" {
            package.to_string()
        } else {
            format!("{}=={}", package, version)
        };

        // Build arguments
        let mut args = vec!["install"];

        // Add extra arguments
        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);

        args.push(&package_spec);

        // Build environment
        let env = self.build_install_env(venv_dir);

        // Run pip install
        let pip_str = pip.to_string_lossy().to_string();
        let output = run_command(&pip_str, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("pip install failed: {}", stderr);
        }

        // Detect executables
        let bin_dir = self.get_bin_dir(venv_dir);
        let mut executables = self.detect_executables(&bin_dir)?;

        // Filter out pip, python, activate scripts
        executables.retain(|e| {
            !e.starts_with("pip")
                && !e.starts_with("python")
                && !e.starts_with("activate")
                && !e.starts_with("Activate")
        });

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "pip".to_string(),
            venv_dir.to_path_buf(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &Path) -> Result<Vec<String>> {
        detect_executables_in_dir(bin_dir)
    }

    fn build_install_env(&self, install_dir: &Path) -> InstallEnv {
        InstallEnv::new()
            .var("VIRTUAL_ENV", install_dir.display().to_string())
            .var("PIP_DISABLE_PIP_VERSION_CHECK", "1")
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        if cfg!(windows) {
            install_dir.join("Scripts")
        } else {
            install_dir.join("bin")
        }
    }

    fn is_available(&self) -> bool {
        self.get_python().is_ok()
    }
}
