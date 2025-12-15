//! PNPM runtime implementation

use crate::config::PnpmUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// PNPM runtime
#[derive(Debug, Clone)]
pub struct PnpmRuntime;

impl PnpmRuntime {
    /// Create a new PNPM runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for PnpmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for PnpmRuntime {
    fn name(&self) -> &str {
        "pnpm"
    }

    fn description(&self) -> &str {
        "Fast, disk space efficient package manager"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Would fetch from GitHub API
        Ok(vec![
            VersionInfo::new("9.0.0"),
            VersionInfo::new("8.15.0").with_lts(true),
        ])
    }

    async fn download_url(&self, version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(PnpmUrlBuilder::download_url(version))
    }
}
