//! Vite runtime implementation
//!
//! Vite is a next-generation frontend build tool that significantly improves
//! the frontend development experience.
//!
//! This runtime installs Vite from npm as an isolated tool, similar to how
//! pipx works for Python packages.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, InstallMethod, InstallResult, PackageRuntime, PathProvider, Platform, Runtime,
    RuntimeContext, VerificationResult, VersionInfo,
};

/// Vite runtime implementation
///
/// Vite is installed as an npm package in an isolated environment,
/// allowing it to be used without a global Node.js installation.
#[derive(Debug, Clone, Default)]
pub struct ViteRuntime;

impl ViteRuntime {
    /// Create a new Vite runtime
    pub fn new() -> Self {
        Self
    }

    /// npm package name for vite
    const PACKAGE_NAME: &'static str = "vite";
}

#[async_trait]
impl Runtime for ViteRuntime {
    fn name(&self) -> &str {
        "vite"
    }

    fn description(&self) -> &str {
        "Vite - Next Generation Frontend Tooling (installed via npm)"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://vitejs.dev/".to_string());
        meta.insert(
            "documentation".to_string(),
            "https://vitejs.dev/guide/".to_string(),
        );
        meta.insert("category".to_string(), "build-tool".to_string());
        meta.insert("install_method".to_string(), "npm".to_string());
        meta.insert("npm_package".to_string(), Self::PACKAGE_NAME.to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // For npm packages, the executable is in the bin directory
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "vite.cmd"
        } else {
            "vite"
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
        let bin_name = if cfg!(windows) { "vite.cmd" } else { "vite" };
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
        let exe_name = if cfg!(windows) { "vite.cmd" } else { "vite" };
        let exe_path = bin_dir.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Vite executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling with: vx install vite".to_string()],
            )
        }
    }
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

#[async_trait]
impl PackageRuntime for ViteRuntime {
    fn install_method(&self) -> InstallMethod {
        InstallMethod::npm(Self::PACKAGE_NAME)
    }

    fn required_runtime(&self) -> &str {
        "node"
    }

    fn required_runtime_version(&self) -> Option<&str> {
        // Vite 5.x requires Node.js 18+
        Some(">=18.0.0")
    }
}
