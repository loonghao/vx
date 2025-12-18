//! VSCode runtime implementations
//!
//! This module provides runtime implementations for:
//! - Visual Studio Code editor

use crate::config::VscodeUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Visual Studio Code runtime
#[derive(Debug, Clone, Default)]
pub struct VscodeRuntime;

impl VscodeRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for VscodeRuntime {
    fn name(&self) -> &str {
        "code"
    }

    fn description(&self) -> &str {
        "Visual Studio Code - Code editing. Redefined."
    }

    fn aliases(&self) -> &[&str] {
        &["vscode", "vs-code", "visual-studio-code"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://code.visualstudio.com/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/microsoft/vscode".to_string(),
        );
        meta.insert("license".to_string(), "MIT".to_string());
        meta.insert("category".to_string(), "editor".to_string());
        meta
    }

    /// VSCode archives have different structures per platform:
    /// - Windows (zip): bin/code.cmd (CLI wrapper) or Code.exe (GUI)
    /// - macOS (zip): Visual Studio Code.app/Contents/Resources/app/bin/code
    /// - Linux (tar.gz): bin/code
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        use vx_runtime::Os;

        match platform.os {
            Os::Windows => "bin/code.cmd".to_string(),
            Os::MacOS => "Visual Studio Code.app/Contents/Resources/app/bin/code".to_string(),
            _ => "bin/code".to_string(),
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch version info from VSCode official API
        // This API returns { "products": [...] } with all platform builds
        let url = "https://code.visualstudio.com/sha";

        let response = ctx
            .get_cached_or_fetch("vscode", || async { ctx.http.get_json_value(url).await })
            .await?;

        let mut versions: Vec<VersionInfo> = Vec::new();
        let mut seen_versions = std::collections::HashSet::new();

        // Parse the response - it's an object with "products" array
        // Each entry has: url, name, version, productVersion, hash, timestamp, sha256hash, build, platform
        let entries = response
            .get("products")
            .and_then(|p| p.as_array())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid response format from VSCode API: expected object with 'products' array"
                )
            })?;

        for entry in entries {
            // Only include stable versions (skip insider builds)
            let build = entry.get("build").and_then(|b| b.as_str()).unwrap_or("");
            if build != "stable" {
                continue;
            }

            // Get product version (e.g., "1.107.1")
            if let Some(version) = entry.get("productVersion").and_then(|v| v.as_str()) {
                // Deduplicate versions (same version appears for multiple platforms)
                if seen_versions.contains(version) {
                    continue;
                }
                seen_versions.insert(version.to_string());

                let timestamp = entry
                    .get("timestamp")
                    .and_then(|t| t.as_i64())
                    .map(|ts| chrono::DateTime::from_timestamp(ts / 1000, 0).unwrap_or_default());

                versions.push(VersionInfo {
                    version: version.to_string(),
                    released_at: timestamp,
                    prerelease: false,
                    lts: false,
                    download_url: None,
                    checksum: None,
                    metadata: HashMap::new(),
                });
            }
        }

        if versions.is_empty() {
            return Err(anyhow::anyhow!(
                "No stable versions found in VSCode API response"
            ));
        }

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(VscodeUrlBuilder::download_url(version, platform))
    }
}
