//! Just runtime implementation
//!
//! Just is a handy way to save and run project-specific commands.
//! https://github.com/casey/just

use crate::config::JustUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

/// Just runtime implementation
#[derive(Debug, Clone, Default)]
pub struct JustRuntime;

impl JustRuntime {
    /// Create a new Just runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for JustRuntime {
    fn name(&self) -> &str {
        "just"
    }

    fn description(&self) -> &str {
        "Just - A handy way to save and run project-specific commands"
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
            "https://github.com/casey/just".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://just.systems/man/en/".to_string(),
        );
        meta.insert("category".to_string(), "command-runner".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // Just extracts directly without subdirectory
        JustUrlBuilder::get_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Just uses plain version numbers without 'v' prefix
        VersionFetcherBuilder::jsdelivr("casey", "just")
            .tool_name("just")
            .prerelease_markers(&["-alpha", "-beta", "-rc", "-dev"])
            .skip_prereleases()
            .limit(50)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(JustUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = JustUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Just executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
