//! Chocolatey runtime implementation

use crate::config::ChocoUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// Chocolatey runtime
#[derive(Debug, Clone)]
pub struct ChocoRuntime;

impl ChocoRuntime {
    /// Create a new Chocolatey runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for ChocoRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for ChocoRuntime {
    fn name(&self) -> &str {
        "choco"
    }

    fn description(&self) -> &str {
        "Chocolatey - The package manager for Windows"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn aliases(&self) -> &[&str] {
        &["chocolatey"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://chocolatey.org/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "windows".to_string());
        meta.insert("category".to_string(), "package-manager".to_string());
        meta
    }

    /// Chocolatey only supports Windows
    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::windows_only()
    }

    /// Chocolatey archives extract to `tools/choco.exe`
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        let dir_name = ChocoUrlBuilder::get_archive_dir_name();
        let exe_name = ChocoUrlBuilder::get_executable_name(platform);
        format!("{}/{}", dir_name, exe_name)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Chocolatey tags are like "2.4.3" (no 'v' prefix)
        ctx.fetch_github_releases(
            "choco",
            "chocolatey",
            "choco",
            GitHubReleaseOptions::new().tag_prefix(""),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Check if platform is supported
        if !self.is_platform_supported(platform) {
            return Ok(None);
        }
        Ok(ChocoUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // Check if platform is supported
        if !self.is_platform_supported(platform) {
            return VerificationResult::failure(
                vec!["Chocolatey is only supported on Windows".to_string()],
                vec!["Use a Windows system to install Chocolatey".to_string()],
            );
        }

        let exe_path = install_path
            .join(ChocoUrlBuilder::get_archive_dir_name())
            .join(ChocoUrlBuilder::get_executable_name(platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Chocolatey executable not found at: {}",
                    exe_path.display()
                )],
                vec![
                    "Try reinstalling Chocolatey".to_string(),
                    "Check if the download completed successfully".to_string(),
                ],
            )
        }
    }
}
