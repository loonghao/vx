//! bat runtime implementation

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{
    platform::Platform, Ecosystem, GitHubReleaseOptions, Runtime, RuntimeContext,
    RuntimeDependency, VersionInfo,
};

/// bat runtime
#[derive(Debug, Clone, Default)]
pub struct BatRuntime;

impl BatRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for BatRuntime {
    fn name(&self) -> &str {
        "bat"
    }

    fn description(&self) -> &str {
        "A cat clone with syntax highlighting and Git integration"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/sharkdp/bat".to_string(),
        );
        meta.insert("license".to_string(), "MIT/Apache-2.0".to_string());
        meta.insert("category".to_string(), "file-viewers".to_string());
        meta
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn dependencies(&self) -> &[RuntimeDependency] {
        &[]
    }

    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        // bat archive structure: bat-v{version}-{target}/bat(.exe)
        let target = crate::config::BatUrlBuilder::get_target_triple(platform).unwrap_or("unknown");
        let exe = crate::config::BatUrlBuilder::get_executable_name(platform);
        format!("bat-v{version}-{target}/{exe}")
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "bat",
            "sharkdp",
            "bat",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(crate::config::BatUrlBuilder::download_url(
            version, platform,
        ))
    }
}
