//! Meson runtime implementation
//!
//! Meson is installed via pip/uv as it's a Python package.
//! https://github.com/mesonbuild/meson

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, InstallMethod, InstallResult, PackageRuntime, PathProvider, Platform, Runtime,
    RuntimeContext, VerificationResult, VersionInfo, compare_semver,
};

/// Meson runtime implementation
///
/// Meson is installed as a pip package in an isolated virtual environment,
/// using uv for faster installation.
#[derive(Debug, Clone, Default)]
pub struct MesonRuntime;

impl MesonRuntime {
    /// Create a new Meson runtime
    pub fn new() -> Self {
        Self
    }

    /// pip package name for meson
    const PACKAGE_NAME: &'static str = "meson";
}

#[async_trait]
impl Runtime for MesonRuntime {
    fn name(&self) -> &str {
        "meson"
    }

    fn description(&self) -> &str {
        "Meson - An extremely fast and user friendly build system"
    }

    fn aliases(&self) -> &[&str] {
        &["mesonbuild"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://mesonbuild.com/".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://mesonbuild.com/Manual.html".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/mesonbuild/meson".to_string(),
        );
        meta.insert("category".to_string(), "build-system".to_string());
        meta.insert("license".to_string(), "Apache-2.0".to_string());
        meta.insert("install_method".to_string(), "pip".to_string());
        meta.insert("pip_package".to_string(), Self::PACKAGE_NAME.to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // For pip packages, the executable is in the venv bin directory
        if platform.is_windows() {
            format!("venv/Scripts/{}", platform.exe_name("meson"))
        } else {
            format!("venv/bin/{}", platform.exe_name("meson"))
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch versions from PyPI
        self.fetch_package_versions(ctx).await
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // pip packages don't have direct download URLs
        Ok(None)
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        // Use pip package installation
        self.install_package(version, ctx).await
    }

    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        let bin_dir = ctx.paths.pip_tool_bin_dir(Self::PACKAGE_NAME, version);
        let exe_name = if cfg!(windows) { "meson.exe" } else { "meson" };
        let exe_path = bin_dir.join(exe_name);
        Ok(exe_path.exists())
    }

    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>> {
        let tool_dir = ctx.paths.pip_tool_dir(Self::PACKAGE_NAME);
        if !ctx.fs.exists(&tool_dir) {
            return Ok(vec![]);
        }

        let entries = ctx.fs.read_dir(&tool_dir)?;
        let mut versions: Vec<String> = entries
            .into_iter()
            .filter(|p| ctx.fs.is_dir(p))
            .filter_map(|p| p.file_name().and_then(|n| n.to_str().map(String::from)))
            .collect();

        // Sort versions (newest first) using simple semver comparison
        versions.sort_by(|a, b| compare_semver(b, a));
        Ok(versions)
    }

    async fn uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let install_dir = ctx.paths.pip_tool_version_dir(Self::PACKAGE_NAME, version);
        if ctx.fs.exists(&install_dir) {
            ctx.fs.remove_dir_all(&install_dir)?;
        }
        Ok(())
    }

    fn verify_installation(
        &self,
        version: &str,
        _install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        // Use package verification instead
        let paths = vx_runtime::RealPathProvider::default();
        let bin_dir = paths.pip_tool_bin_dir(Self::PACKAGE_NAME, version);
        let exe_name = if cfg!(windows) { "meson.exe" } else { "meson" };
        let exe_path = bin_dir.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "meson executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling with: vx install meson".to_string()],
            )
        }
    }
}

#[async_trait]
impl PackageRuntime for MesonRuntime {
    fn install_method(&self) -> InstallMethod {
        InstallMethod::pip(Self::PACKAGE_NAME)
    }

    fn required_runtime(&self) -> &str {
        "uv" // Use uv for faster Python package installation
    }

    fn required_runtime_version(&self) -> Option<&str> {
        None
    }
}
