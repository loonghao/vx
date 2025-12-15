//! Bun runtime implementation

use crate::config::BunUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Bun runtime
#[derive(Debug, Clone)]
pub struct BunRuntime;

impl BunRuntime {
    /// Create a new Bun runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for BunRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for BunRuntime {
    fn name(&self) -> &str {
        "bun"
    }

    fn description(&self) -> &str {
        "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Bun versions - would fetch from GitHub API in production
        Ok(vec![
            VersionInfo::new("1.1.42").with_lts(true), // Latest stable
            VersionInfo::new("1.1.0"),
            VersionInfo::new("1.0.0"),
        ])
    }

    async fn download_url(&self, version: &str, _platform: &Platform) -> Result<Option<String>> {
        let (platform, arch) = BunUrlBuilder::get_platform_string();
        Ok(BunUrlBuilder::download_url(version, platform, arch))
    }
}

/// Bunx runtime (package runner)
#[derive(Debug, Clone)]
pub struct BunxRuntime;

impl BunxRuntime {
    /// Create a new Bunx runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for BunxRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for BunxRuntime {
    fn name(&self) -> &str {
        "bunx"
    }

    fn description(&self) -> &str {
        "Bun package runner (like npx)"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Bunx uses the same versions as Bun
        BunRuntime::new().fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Bunx is part of Bun installation
        BunRuntime::new().download_url(version, platform).await
    }
}
