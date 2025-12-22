//! rcedit runtime implementation
//!
//! rcedit is a command-line tool to edit resources of Windows executables.
//! https://github.com/electron/rcedit

use crate::config::RceditUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, Os, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// rcedit runtime implementation
#[derive(Debug, Clone, Default)]
pub struct RceditRuntime;

impl RceditRuntime {
    /// Create a new rcedit runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for RceditRuntime {
    fn name(&self) -> &str {
        "rcedit"
    }

    fn description(&self) -> &str {
        "rcedit - Command-line tool to edit resources of Windows executables"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/electron/rcedit".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://github.com/electron/rcedit#readme".to_string(),
        );
        meta.insert("category".to_string(), "windows-tools".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // rcedit is a single executable, no subdirectory
        RceditUrlBuilder::get_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = "https://api.github.com/repos/electron/rcedit/releases";
        let response = ctx.http.get_json_value(url).await?;

        let mut versions = Vec::new();

        if let Some(releases) = response.as_array() {
            for release in releases {
                // Skip drafts
                if release
                    .get("draft")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    continue;
                }

                // Check if prerelease
                let is_prerelease = release
                    .get("prerelease")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // Get tag name (version)
                let tag_name = match release.get("tag_name").and_then(|v| v.as_str()) {
                    Some(tag) => tag,
                    None => continue,
                };

                // rcedit uses 'v' prefix in tags, strip it
                let version = tag_name.strip_prefix('v').unwrap_or(tag_name).to_string();

                // Get published date
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
        Ok(RceditUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // rcedit only supports Windows
        if platform.os != Os::Windows {
            return VerificationResult::failure(
                vec!["rcedit is only available for Windows".to_string()],
                vec!["Use a Windows system to install rcedit".to_string()],
            );
        }

        let exe_name = RceditUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "rcedit executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
