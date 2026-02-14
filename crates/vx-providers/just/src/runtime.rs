//! Just runtime implementation

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Runtime, RuntimeContext, RuntimeDependency, VersionInfo,
    layout::{ArchiveLayout, DownloadType, ExecutableLayout},
    platform::Platform,
};

/// Just runtime - a handy way to save and run project-specific commands
#[derive(Debug, Clone, Default)]
pub struct JustRuntime;

impl JustRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for JustRuntime {
    fn name(&self) -> &str {
        "just"
    }

    fn description(&self) -> &str {
        "A handy way to save and run project-specific commands"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Unknown // just is language-agnostic
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/casey/just".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/casey/just".to_string(),
        );
        meta.insert("license".to_string(), "CC0-1.0".to_string());
        meta
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn dependencies(&self) -> &[RuntimeDependency] {
        &[]
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "just",
            "casey",
            "just",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Use the JustUrlBuilder for correct URL construction
        Ok(crate::config::JustUrlBuilder::download_url(
            version, platform,
        ))
    }

    fn executable_layout(&self) -> Option<ExecutableLayout> {
        Some(ExecutableLayout {
            download_type: DownloadType::Archive,
            binary: None,
            archive: Some(ArchiveLayout {
                executable_paths: vec!["just.exe".to_string(), "just".to_string()],
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
