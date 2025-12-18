//! Vite runtime implementation
//!
//! Vite is a next-generation frontend build tool that significantly improves
//! the frontend development experience.
//! <https://github.com/nicholasruunu/vite-standalone>

use crate::config::ViteUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo};

/// Vite runtime implementation
#[derive(Debug, Clone, Default)]
pub struct ViteRuntime;

impl ViteRuntime {
    /// Create a new Vite runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for ViteRuntime {
    fn name(&self) -> &str {
        "vite"
    }

    fn description(&self) -> &str {
        "Vite - Next Generation Frontend Tooling"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://vitejs.dev/".to_string());
        meta.insert(
            "documentation".to_string(),
            "https://vitejs.dev/guide/".to_string(),
        );
        meta.insert("category".to_string(), "build-tool".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        ViteUrlBuilder::get_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = "https://api.github.com/repos/nicholasruunu/vite-standalone/releases";
        let response = ctx.http.get_json_value(url).await?;

        let mut versions = Vec::new();

        if let Some(releases) = response.as_array() {
            for release in releases {
                if release
                    .get("draft")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    continue;
                }

                let is_prerelease = release
                    .get("prerelease")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let tag_name = match release.get("tag_name").and_then(|v| v.as_str()) {
                    Some(tag) => tag,
                    None => continue,
                };

                let version = tag_name.strip_prefix('v').unwrap_or(tag_name).to_string();

                let published_at = release
                    .get("published_at")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let mut version_info = VersionInfo::new(version)
                    .with_lts(false)
                    .with_prerelease(is_prerelease);

                if let Some(date) = published_at {
                    version_info = version_info.with_release_date(date);
                }

                versions.push(version_info);
            }
        }

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(ViteUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = ViteUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Vite executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
