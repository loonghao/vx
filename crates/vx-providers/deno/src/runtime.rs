//! Deno runtime implementation

use crate::config::DenoUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

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
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "deno.exe"
        } else {
            "deno"
        };
        exe_name.to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = "https://api.github.com/repos/denoland/deno/releases?per_page=50";

        let data = ctx
            .get_cached_or_fetch_with_url(self.name(), url, || async {
                ctx.http.get_json_value(url).await
            })
            .await?;

        let versions: Vec<VersionInfo> = data
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format from Deno GitHub API"))?
            .iter()
            .filter_map(|release| {
                let tag = release.get("tag_name")?.as_str()?;
                // Remove 'v' prefix
                let version = tag.strip_prefix('v').unwrap_or(tag);
                let prerelease = release
                    .get("prerelease")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(false);

                Some(VersionInfo::new(version).with_prerelease(prerelease))
            })
            .collect();

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(DenoUrlBuilder::download_url(version, platform))
    }
}
