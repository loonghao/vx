//! Deno runtime implementation

use crate::config::DenoUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

/// Deno runtime
#[derive(Debug, Clone)]
pub struct DenoRuntime;

impl DenoRuntime {
    /// Create a new Deno runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for DenoRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for DenoRuntime {
    fn name(&self) -> &str {
        "deno"
    }

    fn description(&self) -> &str {
        "A secure runtime for JavaScript and TypeScript"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs // Deno is part of the JavaScript ecosystem
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://deno.land/".to_string());
        meta.insert("ecosystem".to_string(), "javascript".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/denoland/deno".to_string(),
        );
        meta.insert("license".to_string(), "MIT".to_string());
        meta
    }

    /// Deno archives extract directly to the binary (no subdirectory)
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        platform.exe_name("deno")
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        VersionFetcherBuilder::jsdelivr("denoland", "deno")
            .tool_name("deno")
            .strip_v_prefix()
            .prerelease_markers(&["canary", "-alpha", "-beta", "-rc"])
            .skip_prereleases()
            .limit(50)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(DenoUrlBuilder::download_url(version, platform))
    }
}
