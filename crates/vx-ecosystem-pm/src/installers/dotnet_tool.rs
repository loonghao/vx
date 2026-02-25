//! dotnet-tool package installer
//!
//! Installs and runs .NET tools via `dotnet tool install`.
//!
//! .NET tools are NuGet packages that contain console applications.
//! They can be installed globally or locally.
//!
//! ## How it works
//!
//! `vx dotnet-tool:dotnet-script` → `dotnet tool install -g dotnet-script`
//! then runs the installed tool directly.

use crate::traits::EcosystemInstaller;
use crate::types::{EcosystemInstallResult, InstallEnv, InstallOptions};
use crate::utils::{detect_executables_in_dir, run_command};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// dotnet tool installer
///
/// Installs .NET tools via `dotnet tool install` into an isolated directory.
#[derive(Debug, Clone, Default)]
pub struct DotnetToolInstaller {
    /// Path to dotnet executable (auto-detected if None)
    dotnet_path: Option<PathBuf>,
}

impl DotnetToolInstaller {
    /// Create a new dotnet tool installer
    pub fn new() -> Self {
        Self { dotnet_path: None }
    }

    /// Create a new dotnet tool installer with a specific dotnet path
    pub fn with_dotnet_path(dotnet_path: PathBuf) -> Self {
        Self {
            dotnet_path: Some(dotnet_path),
        }
    }

    /// Get the dotnet executable path
    fn get_dotnet(&self) -> Result<String> {
        if let Some(ref path) = self.dotnet_path {
            return Ok(path.display().to_string());
        }

        if which::which("dotnet").is_ok() {
            return Ok("dotnet".to_string());
        }

        bail!(
            "dotnet not found in PATH. Please install .NET SDK first (https://dotnet.microsoft.com/download)."
        )
    }
}

#[async_trait]
impl EcosystemInstaller for DotnetToolInstaller {
    fn ecosystem(&self) -> &'static str {
        "dotnet-tool"
    }

    async fn install(
        &self,
        install_dir: &Path,
        package: &str,
        version: &str,
        options: &InstallOptions,
    ) -> Result<EcosystemInstallResult> {
        let dotnet = self.get_dotnet()?;

        // Create installation directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        // Build package spec
        let install_dir_str = install_dir.to_string_lossy().to_string();

        // dotnet tool install --tool-path <dir> <package> [--version <version>]
        let mut args = vec!["tool", "install", "--tool-path", &install_dir_str];

        if options.force {
            // dotnet tool update is used for reinstall
            // We'll handle this by using update instead
        }

        // Add version if specified
        let version_arg;
        if version != "latest" {
            version_arg = version.to_string();
            args.push("--version");
            args.push(&version_arg);
        }

        let extra_args: Vec<&str> = options.extra_args.iter().map(|s| s.as_str()).collect();
        args.extend(extra_args);
        args.push(package);

        let env = self.build_install_env(install_dir);
        let output = run_command(&dotnet, &args, &env, options.verbose)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If already installed, try update
            if stderr.contains("already installed") || options.force {
                let mut update_args = vec!["tool", "update", "--tool-path", &install_dir_str];
                if version != "latest" {
                    update_args.push("--version");
                    update_args.push(version);
                }
                update_args.push(package);

                let update_output = run_command(&dotnet, &update_args, &env, options.verbose)?;
                if !update_output.status.success() {
                    let update_stderr = String::from_utf8_lossy(&update_output.stderr);
                    bail!("dotnet tool install/update failed: {}", update_stderr);
                }
            } else {
                bail!("dotnet tool install failed: {}", stderr);
            }
        }

        // Detect executables in the tool-path directory
        let bin_dir = self.get_bin_dir(install_dir);
        let executables = self.detect_executables(&bin_dir)?;

        Ok(EcosystemInstallResult::new(
            package.to_string(),
            version.to_string(),
            "dotnet-tool".to_string(),
            install_dir.to_path_buf(),
            bin_dir,
        )
        .with_executables(executables))
    }

    fn detect_executables(&self, bin_dir: &Path) -> Result<Vec<String>> {
        detect_executables_in_dir(bin_dir)
    }

    fn build_install_env(&self, _install_dir: &Path) -> InstallEnv {
        InstallEnv::new()
            .var("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
            .var("DOTNET_NOLOGO", "1")
    }

    fn get_bin_dir(&self, install_dir: &Path) -> PathBuf {
        // dotnet tool install --tool-path puts executables directly in the specified dir
        install_dir.to_path_buf()
    }

    fn is_available(&self) -> bool {
        self.get_dotnet().is_ok()
    }
}
