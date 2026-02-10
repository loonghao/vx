//! x-cmd runtime implementation
//!
//! x-cmd is a compact and powerful command-line toolbox.
//!
//! Homepage: https://x-cmd.com
//! Repository: https://github.com/x-cmd/x-cmd

use crate::config::XCmdConfig;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// x-cmd runtime implementation
#[derive(Debug, Clone)]
pub struct XCmdRuntime;

impl XCmdRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for XCmdRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for XCmdRuntime {
    fn name(&self) -> &str {
        "x-cmd"
    }

    fn description(&self) -> &str {
        "x-cmd - Compact and powerful command-line toolbox with AI integration"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn aliases(&self) -> &[&str] {
        &["xcmd", "x_cmd"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://x-cmd.com".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/x-cmd/x-cmd".to_string(),
        );
        meta.insert("category".to_string(), "devtools".to_string());
        meta.insert("license".to_string(), "AGPL-3.0".to_string());
        meta
    }

    /// x-cmd is installed via shell script, no binary archive
    fn executable_relative_path(&self, _version: &str, _platform: &Platform) -> String {
        XCmdConfig::executable_name().to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "x-cmd",
            "x-cmd",
            "x-cmd",
            GitHubReleaseOptions::new().strip_v_prefix(true),
        )
        .await
    }

    /// x-cmd doesn't have a direct download URL - it's installed via shell script
    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // x-cmd is installed via script, not a direct download
        Ok(None)
    }

    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // Check common installation paths
        for search_path in XCmdConfig::search_paths(platform) {
            let x_path = Path::new(search_path).join(XCmdConfig::executable_name());
            if x_path.exists() {
                return VerificationResult::success(x_path);
            }
        }

        // Also check if 'x' is available on PATH via which
        if let Ok(path) = which::which(XCmdConfig::executable_name()) {
            return VerificationResult::success(path);
        }

        VerificationResult::failure(
            vec!["x-cmd not found in common installation paths".to_string()],
            vec![
                format!(
                    "Install x-cmd: {}",
                    XCmdConfig::install_command(platform)
                ),
                "Visit https://x-cmd.com for more information".to_string(),
            ],
        )
    }

    /// Check if x-cmd is already installed on the system
    async fn is_installed(&self, _version: &str, _ctx: &RuntimeContext) -> Result<bool> {
        Ok(which::which(XCmdConfig::executable_name()).is_ok())
    }
}
