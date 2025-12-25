//! Docker runtime implementation

use crate::config::DockerUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VersionInfo};

/// Docker runtime
#[derive(Debug, Clone)]
pub struct DockerRuntime;

impl DockerRuntime {
    /// Create a new Docker runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for DockerRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for DockerRuntime {
    fn name(&self) -> &str {
        "docker"
    }

    fn description(&self) -> &str {
        "Docker CLI - Container runtime command-line interface"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("container".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["docker-cli"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.docker.com/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "container".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/docker/cli".to_string(),
        );
        meta.insert("license".to_string(), "Apache-2.0".to_string());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::all_common()
    }

    /// Docker is extracted from archive to docker/docker
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // Docker archives extract to docker/ directory
        format!("docker/{}", platform.exe_name("docker"))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from docker/cli GitHub releases
        ctx.fetch_github_releases(
            "docker",
            "docker",
            "cli",
            GitHubReleaseOptions::new().strip_v_prefix(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(DockerUrlBuilder::download_url(version, platform))
    }
}
