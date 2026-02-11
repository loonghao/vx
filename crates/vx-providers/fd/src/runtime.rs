//! fd runtime implementation

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{
    platform::Platform, Ecosystem, GitHubReleaseOptions, Runtime, RuntimeContext,
    RuntimeDependency, VersionInfo,
};

/// fd runtime
#[derive(Debug, Clone, Default)]
pub struct FdRuntime;

impl FdRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for FdRuntime {
    fn name(&self) -> &str {
        "fd"
    }

    fn description(&self) -> &str {
        "A simple, fast and user-friendly alternative to find"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/sharkdp/fd".to_string(),
        );
        meta.insert("license".to_string(), "MIT/Apache-2.0".to_string());
        meta.insert("category".to_string(), "search-tools".to_string());
        meta
    }

    fn aliases(&self) -> &[&str] {
        &["fd-find"]
    }

    fn dependencies(&self) -> &[RuntimeDependency] {
        &[]
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "fd",
            "sharkdp",
            "fd",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(crate::config::FdUrlBuilder::download_url(
            version, platform,
        ))
    }
}
