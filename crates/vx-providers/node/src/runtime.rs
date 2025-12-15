//! Node.js runtime implementations
//!
//! This module provides runtime implementations for:
//! - Node.js JavaScript runtime
//! - NPM package manager
//! - NPX package runner

use crate::config::NodeUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, RuntimeDependency, VersionInfo};

/// Node.js JavaScript runtime
#[derive(Debug, Clone, Default)]
pub struct NodeRuntime;

impl NodeRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for NodeRuntime {
    fn name(&self) -> &str {
        "node"
    }

    fn description(&self) -> &str {
        "Node.js JavaScript runtime"
    }

    fn aliases(&self) -> &[&str] {
        &["nodejs"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://nodejs.org/".to_string());
        meta.insert("ecosystem".to_string(), "javascript".to_string());
        meta
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = "https://nodejs.org/dist/index.json";
        let response = ctx.http.get_json_value(url).await?;

        let versions: Vec<VersionInfo> = response
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?
            .iter()
            .filter_map(|v| {
                let version_str = v.get("version")?.as_str()?;
                // Remove 'v' prefix
                let version = version_str.strip_prefix('v').unwrap_or(version_str);
                let lts = v.get("lts").and_then(|l| l.as_str()).is_some();
                let date_str = v.get("date").and_then(|d| d.as_str());
                let released_at = date_str.and_then(|d| {
                    chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                        .ok()
                        .and_then(|date| {
                            date.and_hms_opt(0, 0, 0).map(|dt| {
                                chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc)
                            })
                        })
                });

                Some(VersionInfo {
                    version: version.to_string(),
                    released_at,
                    prerelease: false,
                    lts,
                    download_url: None,
                    checksum: None,
                    metadata: HashMap::new(),
                })
            })
            .collect();

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(NodeUrlBuilder::download_url(version, platform))
    }
}

/// NPM package manager runtime
#[derive(Debug, Clone, Default)]
pub struct NpmRuntime;

impl NpmRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for NpmRuntime {
    fn name(&self) -> &str {
        "npm"
    }

    fn description(&self) -> &str {
        "Node.js package manager"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn dependencies(&self) -> &[RuntimeDependency] {
        // npm depends on node
        static DEPS: &[RuntimeDependency] = &[];
        // Note: We return empty here because dependencies are lazily constructed
        // In a real implementation, we'd use OnceCell or similar
        DEPS
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://www.npmjs.com/".to_string());
        meta.insert("ecosystem".to_string(), "javascript".to_string());
        meta.insert("bundled_with".to_string(), "node".to_string());
        meta
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // NPM is bundled with Node.js, so we fetch Node.js versions
        // and map them to the bundled npm versions
        let node_runtime = NodeRuntime::new();
        node_runtime.fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // NPM is bundled with Node.js
        Ok(NodeUrlBuilder::download_url(version, platform))
    }
}

/// NPX package runner runtime
#[derive(Debug, Clone, Default)]
pub struct NpxRuntime;

impl NpxRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for NpxRuntime {
    fn name(&self) -> &str {
        "npx"
    }

    fn description(&self) -> &str {
        "Node.js package runner"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.npmjs.com/package/npx".to_string(),
        );
        meta.insert("ecosystem".to_string(), "javascript".to_string());
        meta.insert("bundled_with".to_string(), "node".to_string());
        meta
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // NPX is bundled with Node.js/NPM
        let node_runtime = NodeRuntime::new();
        node_runtime.fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // NPX is bundled with Node.js
        Ok(NodeUrlBuilder::download_url(version, platform))
    }
}
