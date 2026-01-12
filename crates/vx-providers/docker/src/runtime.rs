//! Docker runtime implementation

use crate::config::DockerUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

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
        // Docker releases are available from download.docker.com, not GitHub
        // Parse the directory listing to get available versions
        let url = "https://download.docker.com/linux/static/stable/x86_64/";
        
        let html = ctx.http.get(url).await?;
        
        // Parse version from links like: docker-29.1.4.tgz
        let version_regex = regex::Regex::new(r#"docker-(\d+\.\d+\.\d+)\.tgz"#)?;
        
        let mut versions: Vec<VersionInfo> = version_regex
            .captures_iter(&html)
            .filter_map(|cap| {
                let version = cap.get(1)?.as_str().to_string();
                Some(VersionInfo::new(version))
            })
            .collect();
        
        // Remove duplicates and sort by version (newest first)
        versions.sort_by(|a, b| {
            // Parse version parts for comparison
            let parse_version = |v: &str| -> (u32, u32, u32) {
                let parts: Vec<u32> = v.split('.').filter_map(|p| p.parse().ok()).collect();
                (
                    parts.first().copied().unwrap_or(0),
                    parts.get(1).copied().unwrap_or(0),
                    parts.get(2).copied().unwrap_or(0),
                )
            };
            parse_version(&b.version).cmp(&parse_version(&a.version))
        });
        versions.dedup_by(|a, b| a.version == b.version);
        
        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(DockerUrlBuilder::download_url(version, platform))
    }
}
