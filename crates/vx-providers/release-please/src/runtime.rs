//! release-please runtime implementation
//!
//! release-please is a Google tool for automating releases based on
//! conventional commits.
//!
//! This runtime installs release-please from npm as an isolated tool.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    compare_semver, Ecosystem, InstallMethod, InstallResult, PackageRuntime, PathProvider,
    Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// release-please runtime implementation
///
/// release-please is installed as an npm package in an isolated environment,
/// allowing it to be used without a global Node.js installation.
#[derive(Debug, Clone, Default)]
pub struct ReleasePleaseRuntime;

impl ReleasePleaseRuntime {
    /// Create a new release-please runtime
    pub fn new() -> Self {
        Self
    }

    /// npm package name for release-please
    const PACKAGE_NAME: &'static str = "release-please";
}

#[async_trait]
impl Runtime for ReleasePleaseRuntime {
    fn name(&self) -> &str {
        "release-please"
    }

    fn description(&self) -> &str {
        "release-please - Automate releases based on conventional commits (installed via npm)"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/googleapis/release-please".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://github.com/googleapis/release-please#readme".to_string(),
        );
        meta.insert("category".to_string(), "devops".to_string());
        meta.insert("install_method".to_string(), "npm".to_string());
        meta.insert("npm_package".to_string(), Self::PACKAGE_NAME.to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // For npm packages, the executable is in the bin directory
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "release-please.cmd"
        } else {
            "release-please"
        };
        format!("bin/{}", exe_name)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch versions from npm registry
        self.fetch_package_versions(ctx).await
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // npm packages don't have direct download URLs
        Ok(None)
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        // Use npm package installation
        self.install_package(version, ctx).await
    }

    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        let install_dir = ctx.paths.npm_tool_version_dir(Self::PACKAGE_NAME, version);
        let bin_name = if cfg!(windows) {
            "release-please.cmd"
        } else {
            "release-please"
        };
        let exe_path = ctx
            .paths
            .npm_tool_bin_dir(Self::PACKAGE_NAME, version)
            .join(bin_name);
        Ok(install_dir.exists() && exe_path.exists())
    }

    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>> {
        let tool_dir = ctx.paths.npm_tool_dir(Self::PACKAGE_NAME);
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
        let install_dir = ctx.paths.npm_tool_version_dir(Self::PACKAGE_NAME, version);
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
        // Note: This is called with the wrong install_path for npm packages,
        // so we need to construct the correct path using RealPathProvider
        let paths = vx_runtime::RealPathProvider::default();
        let bin_dir = paths.npm_tool_bin_dir(Self::PACKAGE_NAME, version);
        let exe_name = if cfg!(windows) {
            "release-please.cmd"
        } else {
            "release-please"
        };
        let exe_path = bin_dir.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "release-please executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling with: vx install release-please".to_string()],
            )
        }
    }
}

#[async_trait]
impl PackageRuntime for ReleasePleaseRuntime {
    fn install_method(&self) -> InstallMethod {
        InstallMethod::npm(Self::PACKAGE_NAME)
    }

    fn required_runtime(&self) -> &str {
        "node"
    }

    fn required_runtime_version(&self) -> Option<&str> {
        // release-please requires Node.js 18+
        Some(">=18.0.0")
    }
}
