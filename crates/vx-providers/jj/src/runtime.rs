//! Jujutsu (jj) runtime implementation

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Runtime, RuntimeContext, RuntimeDependency, VersionInfo,
    layout::{ArchiveLayout, DownloadType, ExecutableLayout},
    platform::Platform,
};

/// Jujutsu (jj) runtime - A Git-compatible DVCS
#[derive(Debug, Clone, Default)]
pub struct JjRuntime;

impl JjRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for JjRuntime {
    fn name(&self) -> &str {
        "jj"
    }

    fn description(&self) -> &str {
        "A Git-compatible DVCS that is both simple and powerful"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Unknown // jj is a VCS tool, not language-specific
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/jj-vcs/jj".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/jj-vcs/jj".to_string(),
        );
        meta.insert("license".to_string(), "Apache-2.0".to_string());
        meta.insert("category".to_string(), "vcs".to_string());
        meta
    }

    fn aliases(&self) -> &[&str] {
        &["jujutsu"]
    }

    fn dependencies(&self) -> &[RuntimeDependency] {
        &[]
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "jj",
            "jj-vcs",
            "jj",
            GitHubReleaseOptions::new()
                .strip_v_prefix(false)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Use the JjUrlBuilder for correct URL construction
        Ok(crate::config::JjUrlBuilder::download_url(version, platform))
    }

    fn executable_layout(&self) -> Option<ExecutableLayout> {
        Some(ExecutableLayout {
            download_type: DownloadType::Archive,
            binary: None,
            archive: Some(ArchiveLayout {
                executable_paths: vec!["jj.exe".to_string(), "jj".to_string()],
                strip_prefix: Some(String::new()),
                permissions: None,
            }),
            windows: None,
            macos: None,
            linux: None,
            msi: None,
        })
    }
}
