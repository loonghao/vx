//! ripgrep runtime implementation

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{
    platform::Platform, Ecosystem, GitHubReleaseOptions, Runtime, RuntimeContext,
    RuntimeDependency, VersionInfo,
};

/// ripgrep runtime
#[derive(Debug, Clone, Default)]
pub struct RipgrepRuntime;

impl RipgrepRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for RipgrepRuntime {
    fn name(&self) -> &str {
        "rg"
    }

    fn description(&self) -> &str {
        "ripgrep - recursively search directories for a regex pattern"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/BurntSushi/ripgrep".to_string(),
        );
        meta.insert("license".to_string(), "MIT/Unlicense".to_string());
        meta.insert("category".to_string(), "search-tools".to_string());
        meta
    }

    fn aliases(&self) -> &[&str] {
        &["ripgrep"]
    }

    fn dependencies(&self) -> &[RuntimeDependency] {
        &[]
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "rg",
            "BurntSushi",
            "ripgrep",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(crate::config::RipgrepUrlBuilder::download_url(
            version, platform,
        ))
    }
}
