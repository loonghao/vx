//! UV runtime implementations
//!
//! This module provides runtime implementations for:
//! - UV Python package installer
//! - UVX Python application runner

use crate::config::UvUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// UV Python package installer runtime
#[derive(Debug, Clone, Default)]
pub struct UvRuntime;

impl UvRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for UvRuntime {
    fn name(&self) -> &str {
        "uv"
    }

    fn description(&self) -> &str {
        "An extremely fast Python package installer and resolver"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://docs.astral.sh/uv/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/astral-sh/uv".to_string(),
        );
        meta.insert("license".to_string(), "MIT OR Apache-2.0".to_string());
        meta
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from GitHub releases API
        let url = "https://api.github.com/repos/astral-sh/uv/releases";
        let response = ctx.http.get_json_value(url).await?;

        let versions: Vec<VersionInfo> = response
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?
            .iter()
            .filter_map(|release| {
                let tag = release.get("tag_name")?.as_str()?;
                let prerelease = release
                    .get("prerelease")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(false);
                let published_at = release.get("published_at").and_then(|d| d.as_str());
                let released_at = published_at.and_then(|d| {
                    chrono::DateTime::parse_from_rfc3339(d)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                });

                Some(VersionInfo {
                    version: tag.to_string(),
                    released_at,
                    prerelease,
                    lts: false,
                    download_url: None,
                    checksum: None,
                    metadata: HashMap::new(),
                })
            })
            .collect();

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(UvUrlBuilder::download_url(version, platform))
    }
}

/// UVX Python application runner runtime
#[derive(Debug, Clone, Default)]
pub struct UvxRuntime;

impl UvxRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for UvxRuntime {
    fn name(&self) -> &str {
        "uvx"
    }

    fn description(&self) -> &str {
        "Python application runner"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://docs.astral.sh/uv/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert("bundled_with".to_string(), "uv".to_string());
        meta
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // UVX is bundled with UV
        let uv_runtime = UvRuntime::new();
        uv_runtime.fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // UVX is bundled with UV
        Ok(UvUrlBuilder::download_url(version, platform))
    }
}
