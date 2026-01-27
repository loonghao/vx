//! Just runtime implementation

use async_trait::async_trait;
use anyhow::Result;
use vx_runtime::{
    Runtime, RuntimeContext, VersionInfo, Ecosystem, RuntimeDependency,
    GitHubReleaseOptions, platform::Platform,
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

    async fn download_url(
        &self,
        version: &str,
        platform: &Platform,
    ) -> Result<Option<String>> {
        // Use the JustUrlBuilder for correct URL construction
        Ok(crate::config::JustUrlBuilder::download_url(version, platform))
    }
}
