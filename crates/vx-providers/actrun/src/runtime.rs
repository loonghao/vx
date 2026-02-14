//! Actrun runtime implementation
//!
//! Actrun is the runner executable of Actionforge for executing
//! GitHub Actions-compatible workflows locally.
//! https://github.com/actionforge/actrun-cli

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Runtime, RuntimeContext, RuntimeDependency, VersionInfo,
    platform::Platform,
};

/// Actrun runtime - the runner executable of Actionforge
#[derive(Debug, Clone, Default)]
pub struct ActrunRuntime;

impl ActrunRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for ActrunRuntime {
    fn name(&self) -> &str {
        "actrun"
    }

    fn description(&self) -> &str {
        "The runner executable of Actionforge for executing workflows locally"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Unknown
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/actionforge/actrun-cli".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/actionforge/actrun-cli".to_string(),
        );
        meta.insert("license".to_string(), "Actionforge EULA".to_string());
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
            "actrun",
            "actionforge",
            "actrun-cli",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(crate::config::ActrunUrlBuilder::download_url(
            version, platform,
        ))
    }
}
