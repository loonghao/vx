//! GitHub CLI runtime implementation
//!
//! GitHub CLI (gh) is a command-line tool that brings GitHub to your terminal.
//!
//! Homepage: https://cli.github.com/
//! Repository: https://github.com/cli/cli

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

use crate::config::GitHubUrlBuilder;

/// GitHub CLI runtime implementation
#[derive(Debug, Clone, Default)]
pub struct GitHubRuntime;

impl GitHubRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for GitHubRuntime {
    fn name(&self) -> &str {
        "gh"
    }

    fn description(&self) -> &str {
        "GitHub CLI - command line tool for GitHub"
    }

    fn aliases(&self) -> &[&str] {
        &["github"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://cli.github.com/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/cli/cli".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://docs.github.com/en/github-cli".to_string(),
        );
        meta.insert("category".to_string(), "devops".to_string());
        meta.insert("license".to_string(), "MIT".to_string());
        meta
    }

    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        // GitHub CLI archive structure:
        // - Windows .zip: bin/gh.exe (no directory prefix)
        // - macOS .zip: gh_2.85.0_macOS_amd64/bin/gh (with directory prefix)
        // - Linux .tar.gz: gh_2.85.0_linux_amd64/bin/gh (with directory prefix)

        let platform_dir = match (&platform.os, &platform.arch) {
            (vx_runtime::Os::Windows, vx_runtime::Arch::X86_64) => "windows_amd64",
            (vx_runtime::Os::Windows, vx_runtime::Arch::Aarch64) => "windows_arm64",
            (vx_runtime::Os::MacOS, vx_runtime::Arch::X86_64) => "macOS_amd64",
            (vx_runtime::Os::MacOS, vx_runtime::Arch::Aarch64) => "macOS_arm64",
            (vx_runtime::Os::Linux, vx_runtime::Arch::X86_64) => "linux_amd64",
            (vx_runtime::Os::Linux, vx_runtime::Arch::Aarch64) => "linux_arm64",
            _ => "unknown",
        };

        match platform.os {
            vx_runtime::Os::Windows => "bin/gh.exe".to_string(),
            _ => format!(
                "gh_{version}_{platform_dir}/bin/gh",
                version = version,
                platform_dir = platform_dir
            ),
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // GitHub CLI uses 'v' prefix in tags
        VersionFetcherBuilder::github_releases("cli", "cli")
            .tool_name("gh")
            .strip_v_prefix()
            .skip_prereleases()
            .limit(50)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(Some(GitHubUrlBuilder::download_url(version, platform)))
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
                vec!["Executable not found at expected location".to_string()],
                vec!["Check download URL and extraction process".to_string()],
            )
        }
    }
}
