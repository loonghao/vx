//! Package-based runtime support (npm/pip packages)
//!
//! This module provides support for tools that are installed via package managers
//! (npm, pip) rather than as standalone binaries.

use crate::context::RuntimeContext;
use crate::platform::Platform;
use crate::runtime::{Runtime, VerificationResult};
use crate::types::{InstallResult, VersionInfo};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Installation method for a runtime
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum InstallMethod {
    /// Standalone binary download (default method)
    #[default]
    Binary,
    /// npm package installation
    NpmPackage {
        /// npm package name (e.g., "vite", "@angular/cli")
        package_name: String,
        /// Binary name if different from package name (e.g., "ng" for @angular/cli)
        bin_name: Option<String>,
    },
    /// pip package installation
    PipPackage {
        /// pip package name (e.g., "rez", "black")
        package_name: String,
        /// Binary name if different from package name
        bin_name: Option<String>,
    },
}

impl InstallMethod {
    /// Create a new npm package install method
    pub fn npm(package_name: impl Into<String>) -> Self {
        Self::NpmPackage {
            package_name: package_name.into(),
            bin_name: None,
        }
    }

    /// Create a new npm package install method with custom bin name
    pub fn npm_with_bin(package_name: impl Into<String>, bin_name: impl Into<String>) -> Self {
        Self::NpmPackage {
            package_name: package_name.into(),
            bin_name: Some(bin_name.into()),
        }
    }

    /// Create a new pip package install method
    pub fn pip(package_name: impl Into<String>) -> Self {
        Self::PipPackage {
            package_name: package_name.into(),
            bin_name: None,
        }
    }

    /// Create a new pip package install method with custom bin name
    pub fn pip_with_bin(package_name: impl Into<String>, bin_name: impl Into<String>) -> Self {
        Self::PipPackage {
            package_name: package_name.into(),
            bin_name: Some(bin_name.into()),
        }
    }

    /// Check if this is an npm package
    pub fn is_npm(&self) -> bool {
        matches!(self, Self::NpmPackage { .. })
    }

    /// Check if this is a pip package
    pub fn is_pip(&self) -> bool {
        matches!(self, Self::PipPackage { .. })
    }

    /// Check if this is a binary download
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Binary)
    }

    /// Get the package name if this is a package-based install
    pub fn package_name(&self) -> Option<&str> {
        match self {
            Self::NpmPackage { package_name, .. } => Some(package_name),
            Self::PipPackage { package_name, .. } => Some(package_name),
            Self::Binary => None,
        }
    }

    /// Get the binary name for this install method
    pub fn bin_name(&self, default_name: &str) -> String {
        match self {
            Self::NpmPackage { bin_name, .. } | Self::PipPackage { bin_name, .. } => {
                bin_name.clone().unwrap_or_else(|| default_name.to_string())
            }
            Self::Binary => default_name.to_string(),
        }
    }
}

/// Trait for package-based runtimes (npm/pip packages)
///
/// This trait extends `Runtime` with package-specific functionality.
/// Runtimes implementing this trait will be installed via npm or pip
/// instead of downloading standalone binaries.
#[async_trait]
pub trait PackageRuntime: Runtime {
    /// Get the installation method for this runtime
    fn install_method(&self) -> InstallMethod;

    /// Get the required runtime for this package
    ///
    /// For npm packages, this typically returns "node".
    /// For pip packages, this typically returns "python" or "uv".
    fn required_runtime(&self) -> &str {
        match self.install_method() {
            InstallMethod::NpmPackage { .. } => "node",
            InstallMethod::PipPackage { .. } => "uv", // Use uv for Python packages
            InstallMethod::Binary => "",
        }
    }

    /// Get the minimum required version of the runtime
    fn required_runtime_version(&self) -> Option<&str> {
        None
    }

    /// Fetch versions from the package registry
    ///
    /// For npm packages, this queries the npm registry.
    /// For pip packages, this queries PyPI.
    async fn fetch_package_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        match self.install_method() {
            InstallMethod::NpmPackage {
                ref package_name, ..
            } => fetch_npm_versions(package_name, ctx).await,
            InstallMethod::PipPackage {
                ref package_name, ..
            } => fetch_pypi_versions(package_name, ctx).await,
            InstallMethod::Binary => {
                // Fall back to the default implementation
                Ok(vec![])
            }
        }
    }

    /// Install the package
    async fn install_package(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        match self.install_method() {
            InstallMethod::NpmPackage {
                ref package_name,
                ref bin_name,
            } => {
                install_npm_package(
                    package_name,
                    bin_name.as_deref().unwrap_or(self.name()),
                    version,
                    ctx,
                )
                .await
            }
            InstallMethod::PipPackage {
                ref package_name,
                ref bin_name,
            } => {
                install_pip_package(
                    package_name,
                    bin_name.as_deref().unwrap_or(self.name()),
                    version,
                    ctx,
                )
                .await
            }
            InstallMethod::Binary => {
                // Fall back to the default binary installation
                Err(anyhow::anyhow!(
                    "Binary installation should use the default Runtime::install method"
                ))
            }
        }
    }

    /// Verify package installation
    fn verify_package_installation(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> VerificationResult {
        let install_method = self.install_method();
        let bin_name = install_method.bin_name(self.name());

        let exe_path = match &install_method {
            InstallMethod::NpmPackage { package_name, .. } => {
                let bin_dir = ctx.paths.npm_tool_bin_dir(package_name, version);
                let exe_name = if cfg!(windows) {
                    format!("{}.cmd", bin_name)
                } else {
                    bin_name
                };
                bin_dir.join(exe_name)
            }
            InstallMethod::PipPackage { package_name, .. } => {
                let bin_dir = ctx.paths.pip_tool_bin_dir(package_name, version);
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", bin_name)
                } else {
                    bin_name
                };
                bin_dir.join(exe_name)
            }
            InstallMethod::Binary => {
                return VerificationResult::failure(
                    vec!["Binary installation should use the default verification".to_string()],
                    vec![],
                );
            }
        };

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Package executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the package".to_string()],
            )
        }
    }
}

/// Fetch versions from npm registry
async fn fetch_npm_versions(package_name: &str, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    let url = format!("https://registry.npmjs.org/{}", package_name);
    let response = ctx.http.get_json_value(&url).await?;

    let mut versions = Vec::new();

    if let Some(versions_obj) = response.get("versions").and_then(|v| v.as_object()) {
        for (version, _) in versions_obj {
            let mut version_info = VersionInfo::new(version.clone());

            // Check if it's a prerelease
            if version.contains('-') {
                version_info = version_info.with_prerelease(true);
            }

            versions.push(version_info);
        }
    }

    // Sort versions (newest first) using simple semver comparison
    versions.sort_by(|a, b| compare_semver(&b.version, &a.version));

    Ok(versions)
}

/// Fetch versions from PyPI
async fn fetch_pypi_versions(package_name: &str, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    let url = format!("https://pypi.org/pypi/{}/json", package_name);
    let response = ctx.http.get_json_value(&url).await?;

    let mut versions = Vec::new();

    if let Some(releases) = response.get("releases").and_then(|v| v.as_object()) {
        for (version, _) in releases {
            let mut version_info = VersionInfo::new(version.clone());

            // Check for prerelease markers
            if version.contains("a")
                || version.contains("b")
                || version.contains("rc")
                || version.contains("dev")
            {
                version_info = version_info.with_prerelease(true);
            }

            versions.push(version_info);
        }
    }

    // Sort versions (newest first) using simple semver comparison
    versions.sort_by(|a, b| compare_semver(&b.version, &a.version));

    Ok(versions)
}

/// Simple semver comparison for sorting versions
fn compare_semver(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_version = |v: &str| -> Vec<u64> {
        v.split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    };

    let a_parts = parse_version(a);
    let b_parts = parse_version(b);

    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        match a_part.cmp(b_part) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    a_parts.len().cmp(&b_parts.len())
}

/// Install an npm package to an isolated environment
async fn install_npm_package(
    package_name: &str,
    bin_name: &str,
    version: &str,
    ctx: &RuntimeContext,
) -> Result<InstallResult> {
    use tracing::{debug, info};

    let install_dir = ctx.paths.npm_tool_version_dir(package_name, version);
    let bin_dir = ctx.paths.npm_tool_bin_dir(package_name, version);

    // Check if already installed
    let exe_name = if cfg!(windows) {
        format!("{}.cmd", bin_name)
    } else {
        bin_name.to_string()
    };
    let exe_path = bin_dir.join(&exe_name);

    if exe_path.exists() {
        debug!("npm package already installed: {}", exe_path.display());
        return Ok(InstallResult::already_installed(
            install_dir,
            exe_path,
            version.to_string(),
        ));
    }

    info!("Installing npm package {}@{}", package_name, version);

    // Create install directory
    ctx.fs.create_dir_all(&install_dir)?;

    // Find node executable - first try to use vx-managed node, then fall back to system
    let node_exe = find_runtime_executable("node", ctx).await?;
    let npm_exe = find_runtime_executable("npm", ctx).await?;

    debug!("Using node: {}", node_exe.display());
    debug!("Using npm: {}", npm_exe.display());

    // Initialize package.json
    let package_json = install_dir.join("package.json");
    if !package_json.exists() {
        let init_content = format!(
            r#"{{"name": "vx-{}-env", "version": "1.0.0", "private": true}}"#,
            package_name.replace('/', "-").replace('@', "")
        );
        std::fs::write(&package_json, init_content)?;
    }

    // Run npm install
    let install_spec = format!("{}@{}", package_name, version);
    let status = std::process::Command::new(&npm_exe)
        .args(["install", "--save", "--silent", &install_spec])
        .current_dir(&install_dir)
        .stdin(std::process::Stdio::null())
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Failed to install npm package {}@{}",
            package_name,
            version
        ));
    }

    // Create bin directory and shim scripts
    ctx.fs.create_dir_all(&bin_dir)?;

    // Find the actual binary in node_modules/.bin
    let node_modules_bin = install_dir.join("node_modules").join(".bin");
    let source_bin = if cfg!(windows) {
        node_modules_bin.join(format!("{}.cmd", bin_name))
    } else {
        node_modules_bin.join(bin_name)
    };

    if source_bin.exists() {
        // Create a wrapper script that sets up the environment
        create_npm_shim(&exe_path, &source_bin, &node_exe)?;
    } else {
        return Err(anyhow::anyhow!(
            "Binary '{}' not found in node_modules/.bin after installing {}",
            bin_name,
            package_name
        ));
    }

    info!(
        "Successfully installed {}@{} to {}",
        package_name,
        version,
        install_dir.display()
    );

    Ok(InstallResult::success(
        install_dir,
        exe_path,
        version.to_string(),
    ))
}

/// Install a pip package to an isolated virtual environment
async fn install_pip_package(
    package_name: &str,
    bin_name: &str,
    version: &str,
    ctx: &RuntimeContext,
) -> Result<InstallResult> {
    use tracing::{debug, info, warn};

    let install_dir = ctx.paths.pip_tool_version_dir(package_name, version);
    let venv_dir = ctx.paths.pip_tool_venv_dir(package_name, version);
    let bin_dir = ctx.paths.pip_tool_bin_dir(package_name, version);

    // Check if already installed
    let exe_name = if cfg!(windows) {
        format!("{}.exe", bin_name)
    } else {
        bin_name.to_string()
    };
    let exe_path = bin_dir.join(&exe_name);

    if exe_path.exists() {
        debug!("pip package already installed: {}", exe_path.display());
        return Ok(InstallResult::already_installed(
            install_dir,
            exe_path,
            version.to_string(),
        ));
    }

    info!("Installing pip package {}=={}", package_name, version);

    // Create install directory
    ctx.fs.create_dir_all(&install_dir)?;

    // Try uv first, but fall back to system Python if it fails
    // (uv has known issues on some Windows configurations)
    let uv_result = if let Ok(uv_exe) = find_runtime_executable("uv", ctx).await {
        debug!("Trying uv: {}", uv_exe.display());
        install_with_uv(&uv_exe, package_name, version, &venv_dir, &bin_dir).await
    } else {
        Err(anyhow::anyhow!("uv not found"))
    };

    if let Err(uv_err) = uv_result {
        warn!(
            "uv installation failed ({}), falling back to system Python",
            uv_err
        );

        // Clean up any partial installation from uv
        if venv_dir.exists() {
            let _ = std::fs::remove_dir_all(&venv_dir);
        }

        // Fall back to system Python
        install_with_system_python(package_name, version, &venv_dir, &bin_dir).await?;
    }

    if !exe_path.exists() {
        return Err(anyhow::anyhow!(
            "Binary '{}' not found after installing {}",
            bin_name,
            package_name
        ));
    }

    info!(
        "Successfully installed {}=={} to {}",
        package_name,
        version,
        install_dir.display()
    );

    Ok(InstallResult::success(
        install_dir,
        exe_path,
        version.to_string(),
    ))
}

/// Install pip package using uv
async fn install_with_uv(
    uv_exe: &Path,
    package_name: &str,
    version: &str,
    venv_dir: &Path,
    bin_dir: &Path,
) -> Result<()> {
    use tracing::debug;

    // Python executable path in venv
    let venv_python = if cfg!(windows) {
        venv_dir.join("Scripts").join("python.exe")
    } else {
        venv_dir.join("bin").join("python")
    };

    // Create venv with uv
    let status = std::process::Command::new(uv_exe)
        .args(["venv", "--quiet", venv_dir.to_str().unwrap()])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("uv venv creation failed"));
    }

    // Install package with uv pip
    let install_spec = format!("{}=={}", package_name, version);
    let status = std::process::Command::new(uv_exe)
        .args([
            "pip",
            "install",
            "--quiet",
            "--python",
            venv_python.to_str().unwrap(),
            &install_spec,
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("uv pip install failed"));
    }

    // Verify the binary exists
    let exe_name = if cfg!(windows) {
        format!("{}.exe", package_name)
    } else {
        package_name.to_string()
    };

    if !bin_dir.join(&exe_name).exists() {
        return Err(anyhow::anyhow!("binary not created by uv"));
    }

    debug!("Successfully installed with uv");
    Ok(())
}

/// Install pip package using system Python
async fn install_with_system_python(
    package_name: &str,
    version: &str,
    venv_dir: &Path,
    _bin_dir: &Path,
) -> Result<()> {
    use tracing::debug;

    let python_exe = find_system_python()?;
    debug!("Using system python: {}", python_exe.display());

    // Create venv
    let status = std::process::Command::new(&python_exe)
        .args(["-m", "venv", venv_dir.to_str().unwrap()])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Failed to create venv at {}",
            venv_dir.display()
        ));
    }

    // Use python -m pip instead of pip directly (more reliable)
    let venv_python = if cfg!(windows) {
        venv_dir.join("Scripts").join("python.exe")
    } else {
        venv_dir.join("bin").join("python")
    };

    // Install package
    let install_spec = format!("{}=={}", package_name, version);
    let status = std::process::Command::new(&venv_python)
        .args(["-m", "pip", "install", "--quiet", &install_spec])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Failed to install pip package {}=={}",
            package_name,
            version
        ));
    }

    debug!("Successfully installed with system Python");
    Ok(())
}

/// Find a runtime executable (vx-managed or system)
async fn find_runtime_executable(
    runtime_name: &str,
    ctx: &RuntimeContext,
) -> Result<std::path::PathBuf> {
    // First, check if we have a vx-managed version
    let runtime_dir = ctx.paths.runtime_store_dir(runtime_name);
    if runtime_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
            // Find the first installed version
            for entry in entries.filter_map(|e| e.ok()) {
                let version_dir = entry.path();
                if version_dir.is_dir() {
                    // Try common executable locations
                    let _platform = Platform::current();
                    let exe_name = if cfg!(windows) {
                        format!("{}.exe", runtime_name)
                    } else {
                        runtime_name.to_string()
                    };

                    // Try bin/{name}
                    let exe_path = version_dir.join("bin").join(&exe_name);
                    if exe_path.exists() {
                        return Ok(exe_path);
                    }

                    // Try {name} directly (for some tools)
                    let exe_path = version_dir.join(&exe_name);
                    if exe_path.exists() {
                        return Ok(exe_path);
                    }

                    // Search recursively
                    if let Some(found) = search_executable(&version_dir, &exe_name, 0, 3) {
                        return Ok(found);
                    }
                }
            }
        }
    }

    // Fall back to system PATH
    which::which(runtime_name)
        .map_err(|_| anyhow::anyhow!("Could not find '{}' in PATH or vx store", runtime_name))
}

/// Find system Python
fn find_system_python() -> Result<std::path::PathBuf> {
    // Try python3 first, then python
    which::which("python3")
        .or_else(|_| which::which("python"))
        .map_err(|_| anyhow::anyhow!("Could not find Python in PATH"))
}

/// Search for an executable in a directory tree
fn search_executable(
    dir: &Path,
    exe_name: &str,
    current_depth: usize,
    max_depth: usize,
) -> Option<std::path::PathBuf> {
    if current_depth > max_depth || !dir.exists() {
        return None;
    }

    let entries = std::fs::read_dir(dir).ok()?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == exe_name {
                    return Some(path);
                }
            }
        } else if path.is_dir() {
            if let Some(found) = search_executable(&path, exe_name, current_depth + 1, max_depth) {
                return Some(found);
            }
        }
    }

    None
}

/// Create an npm shim script
fn create_npm_shim(shim_path: &Path, source_bin: &Path, _node_exe: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        // On Windows, create a .cmd wrapper
        let content = format!("@echo off\r\n\"{}\"\r\n", source_bin.display());
        std::fs::write(shim_path, content)?;
    }

    #[cfg(not(windows))]
    {
        // On Unix, create a symlink or shell script
        if std::os::unix::fs::symlink(source_bin, shim_path).is_err() {
            // Fall back to shell script if symlink fails
            let content = format!("#!/bin/sh\nexec \"{}\" \"$@\"\n", source_bin.display());
            std::fs::write(shim_path, content)?;

            // Make executable
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(shim_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(shim_path, perms)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_method_npm() {
        let method = InstallMethod::npm("vite");
        assert!(method.is_npm());
        assert!(!method.is_pip());
        assert_eq!(method.package_name(), Some("vite"));
        assert_eq!(method.bin_name("vite"), "vite");
    }

    #[test]
    fn test_install_method_npm_with_bin() {
        let method = InstallMethod::npm_with_bin("@angular/cli", "ng");
        assert!(method.is_npm());
        assert_eq!(method.package_name(), Some("@angular/cli"));
        assert_eq!(method.bin_name("angular"), "ng");
    }

    #[test]
    fn test_install_method_pip() {
        let method = InstallMethod::pip("rez");
        assert!(method.is_pip());
        assert!(!method.is_npm());
        assert_eq!(method.package_name(), Some("rez"));
    }
}
