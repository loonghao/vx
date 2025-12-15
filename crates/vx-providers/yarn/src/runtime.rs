//! Yarn runtime implementation

use crate::config::YarnUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Yarn runtime
#[derive(Debug, Clone)]
pub struct YarnRuntime;

impl YarnRuntime {
    /// Create a new Yarn runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for YarnRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for YarnRuntime {
    fn name(&self) -> &str {
        "yarn"
    }

    fn description(&self) -> &str {
        "Fast, reliable, and secure dependency management"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Yarn 1.x is recommended for vx as it uses tar.gz archives
        // Yarn 2+ (Berry) uses a single .js file which requires different handling
        Ok(vec![
            VersionInfo::new("1.22.22").with_lts(true), // Latest Yarn 1.x (stable)
            VersionInfo::new("1.22.21"),
            VersionInfo::new("1.22.19"),
        ])
    }

    async fn download_url(&self, version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(YarnUrlBuilder::download_url(version))
    }
}
