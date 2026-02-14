//! fzf runtime implementation

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Runtime, RuntimeContext, RuntimeDependency, VersionInfo,
    platform::Platform,
};

/// fzf runtime
#[derive(Debug, Clone, Default)]
pub struct FzfRuntime;

impl FzfRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for FzfRuntime {
    fn name(&self) -> &str {
        "fzf"
    }

    fn description(&self) -> &str {
        "A command-line fuzzy finder"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/junegunn/fzf".to_string(),
        );
        meta.insert("license".to_string(), "MIT".to_string());
        meta.insert("category".to_string(), "search-tools".to_string());
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
            "fzf",
            "junegunn",
            "fzf",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(crate::config::FzfUrlBuilder::download_url(
            version, platform,
        ))
    }
}
