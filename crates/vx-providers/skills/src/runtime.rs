//! Skills runtime implementation
//!
//! Skills is the open agent skills tool by Vercel Labs for managing
//! AI coding agent skills across multiple AI assistants.
//!
//! This runtime installs Skills from npm as an isolated tool.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    compare_semver, Ecosystem, InstallMethod, InstallResult, PackageRuntime, PathProvider,
    Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// Skills runtime implementation
///
/// Skills is installed as an npm package in an isolated environment.
#[derive(Debug, Clone, Default)]
pub struct SkillsRuntime;

impl SkillsRuntime {
    /// Create a new Skills runtime
    pub fn new() -> Self {
        Self
    }

    /// npm package name for skills
    const PACKAGE_NAME: &'static str = "skills";
}

#[async_trait]
impl Runtime for SkillsRuntime {
    fn name(&self) -> &str {
        "skills"
    }

    fn description(&self) -> &str {
        "Vercel Skills - The open agent skills tool (installed via npm)"
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
            "https://github.com/vercel-labs/skills".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://github.com/vercel-labs/skills#readme".to_string(),
        );
        meta.insert("category".to_string(), "ai".to_string());
        meta.insert("install_method".to_string(), "npm".to_string());
        meta.insert("npm_package".to_string(), Self::PACKAGE_NAME.to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "skills.cmd"
        } else {
            "skills"
        };
        format!("bin/{}", exe_name)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        self.fetch_package_versions(ctx).await
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // npm packages don't have direct download URLs
        Ok(None)
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        self.install_package(version, ctx).await
    }

    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        let install_dir = ctx.paths.npm_tool_version_dir(Self::PACKAGE_NAME, version);
        let bin_name = if cfg!(windows) {
            "skills.cmd"
        } else {
            "skills"
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
        let paths = vx_runtime::RealPathProvider::default();
        let bin_dir = paths.npm_tool_bin_dir(Self::PACKAGE_NAME, version);
        let exe_name = if cfg!(windows) {
            "skills.cmd"
        } else {
            "skills"
        };
        let exe_path = bin_dir.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Skills executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling with: vx install skills".to_string()],
            )
        }
    }
}

#[async_trait]
impl PackageRuntime for SkillsRuntime {
    fn install_method(&self) -> InstallMethod {
        InstallMethod::npm(Self::PACKAGE_NAME)
    }

    fn required_runtime(&self) -> &str {
        "node"
    }

    fn required_runtime_version(&self) -> Option<&str> {
        Some(">=18.0.0")
    }
}
