//! .NET SDK runtime implementation
//!
//! .NET is a free, cross-platform, open-source developer platform for building
//! many different types of applications including web, mobile, desktop, games, and IoT.
//!
//! Homepage: https://dotnet.microsoft.com

use crate::config::DotnetUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo};

/// .NET SDK runtime implementation
#[derive(Debug, Clone, Default)]
pub struct DotnetRuntime;

impl DotnetRuntime {
    /// Create a new .NET SDK runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for DotnetRuntime {
    fn name(&self) -> &str {
        "dotnet"
    }

    fn description(&self) -> &str {
        ".NET SDK - Free, cross-platform, open-source developer platform"
    }

    fn aliases(&self) -> &[&str] {
        &["dotnet-sdk"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://dotnet.microsoft.com".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://learn.microsoft.com/dotnet".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/dotnet/sdk".to_string(),
        );
        meta.insert("category".to_string(), "sdk".to_string());
        meta.insert("license".to_string(), "MIT".to_string());
        meta
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from .NET releases index with caching
        let url = "https://raw.githubusercontent.com/dotnet/core/main/release-notes/releases-index.json";

        let response = ctx
            .get_cached_or_fetch("dotnet", || async { ctx.http.get_json_value(url).await })
            .await?;

        let releases = response
            .get("releases-index")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                anyhow::anyhow!("Invalid response format from .NET releases index")
            })?;

        let mut versions = Vec::new();

        for release in releases {
            let channel_version = release
                .get("channel-version")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            let latest_sdk = release
                .get("latest-sdk")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            let support_phase = release
                .get("support-phase")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let release_type = release
                .get("release-type")
                .and_then(|v| v.as_str())
                .unwrap_or("sts");

            let eol_date = release
                .get("eol-date")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Skip EOL versions unless specifically requested
            if support_phase == "eol" {
                debug!("Skipping EOL version: {}", channel_version);
                continue;
            }

            // Only include active/supported versions
            if !latest_sdk.is_empty() && support_phase == "active" {
                let is_lts = release_type == "lts";

                let mut metadata = HashMap::new();
                metadata.insert("channel".to_string(), channel_version.to_string());
                metadata.insert("support_phase".to_string(), support_phase.to_string());
                metadata.insert("release_type".to_string(), release_type.to_string());
                if let Some(ref eol) = eol_date {
                    metadata.insert("eol_date".to_string(), eol.clone());
                }

                versions.push(VersionInfo {
                    version: latest_sdk.to_string(),
                    released_at: None,
                    prerelease: false,
                    lts: is_lts,
                    download_url: None,
                    checksum: None,
                    metadata,
                });
            }
        }

        Ok(versions)
    }

    /// .NET SDK archives extract contents directly
    /// e.g., dotnet-sdk-9.0.310-win-x64.zip extracts to: dotnet.exe
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        platform.exe_name("dotnet").to_string()
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(DotnetUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = DotnetUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    ".NET SDK executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
