//! Git runtime implementation.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

use crate::config::GitUrlBuilder;

/// Git runtime for managing Git installations.
#[derive(Debug, Default)]
pub struct GitRuntime;

impl GitRuntime {
    /// Create a new Git runtime instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for GitRuntime {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "Git - Distributed version control system"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("homepage".to_string(), "https://git-scm.com/".to_string());
        metadata.insert(
            "repository".to_string(),
            "https://github.com/git/git".to_string(),
        );
        metadata.insert(
            "documentation".to_string(),
            "https://git-scm.com/doc".to_string(),
        );
        metadata
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch versions from Git for Windows releases
        // This provides portable Git versions for Windows
        // Note: Git for Windows uses tags like "v2.52.0.windows.1", we extract just "2.52.0"
        let mut versions = ctx
            .fetch_github_releases(
                "git",
                "git-for-windows",
                "git",
                GitHubReleaseOptions::new()
                    .strip_v_prefix(true)
                    .skip_prereleases(true)
                    .per_page(50),
            )
            .await?;

        // Transform versions: "2.52.0.windows.1" -> "2.52.0"
        for v in &mut versions {
            if let Some(base_version) = v.version.split(".windows.").next() {
                v.version = base_version.to_string();
            }
        }

        // Deduplicate versions (multiple .windows.X releases for same base version)
        let mut seen = std::collections::HashSet::new();
        versions.retain(|v| seen.insert(v.version.clone()));

        Ok(versions)
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        // We only support installing Git via portable MinGit downloads on Windows.
        // On Linux/macOS, users should install Git via their system package manager.
        Platform::windows_only()
    }

    fn check_platform_support(&self) -> Result<(), String> {
        let current = Platform::current();
        if self.is_platform_supported(&current) {
            return Ok(());
        }

        Err(
            "Git installs via vx are only supported on Windows (portable MinGit).\n\nOn macOS/Linux, please install Git via your system package manager:\n  - macOS: brew install git\n  - Ubuntu/Debian: sudo apt install git\n  - Fedora/RHEL: sudo dnf install git\n  - Arch: sudo pacman -S git"
                .to_string(),
        )
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(GitUrlBuilder::download_url(version, platform))
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // MinGit extracts to a flat structure with cmd/git.exe
        match platform.os {
            vx_runtime::Os::Windows => "cmd/git.exe".to_string(),
            _ => "bin/git".to_string(),
        }
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path(version, platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Git executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec![
                    "Try reinstalling Git with: vx install git".to_string(),
                    "Check if the download completed successfully".to_string(),
                ],
            )
        }
    }
}
