//! Yarn runtime implementation

use crate::config::YarnUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use vx_runtime::{Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VersionInfo};

/// Yarn runtime
#[derive(Debug, Clone)]
pub struct YarnRuntime;

impl YarnRuntime {
    /// Create a new Yarn runtime
    pub fn new() -> Self {
        Self
    }

    /// Get the directory name inside the archive for a given version
    fn get_archive_dir_name(version: &str) -> String {
        format!("yarn-v{}", version)
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

    /// Yarn 1.x archives extract to `yarn-v{version}/bin/yarn`
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        let dir_name = Self::get_archive_dir_name(version);
        // Yarn uses .cmd on Windows
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "yarn.cmd"
        } else {
            "yarn"
        };
        format!("{}/bin/{}", dir_name, exe_name)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Yarn 1.x uses yarnpkg/yarn repo
        // Mark 1.22.x as LTS (latest stable Yarn 1.x)
        ctx.fetch_github_releases(
            "yarn",
            "yarnpkg",
            "yarn",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .lts_detector(|v| v.starts_with("1.22.")),
        )
        .await
    }

    async fn download_url(&self, version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(YarnUrlBuilder::download_url(version))
    }
}
