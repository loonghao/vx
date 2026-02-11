//! Starship runtime implementation

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{
    platform::Platform, Ecosystem, GitHubReleaseOptions, Runtime, RuntimeContext,
    RuntimeDependency, VersionInfo,
};

/// Starship runtime
#[derive(Debug, Clone, Default)]
pub struct StarshipRuntime;

impl StarshipRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for StarshipRuntime {
    fn name(&self) -> &str {
        "starship"
    }

    fn description(&self) -> &str {
        "The minimal, blazing-fast, and infinitely customizable prompt for any shell"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://starship.rs".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/starship/starship".to_string(),
        );
        meta.insert("license".to_string(), "ISC".to_string());
        meta.insert("category".to_string(), "shell-prompt".to_string());
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
            "starship",
            "starship",
            "starship",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(crate::config::StarshipUrlBuilder::download_url(
            version, platform,
        ))
    }
}
