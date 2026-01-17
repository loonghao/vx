//! Ninja runtime implementation
//!
//! Ninja is a small build system with a focus on speed.
//! https://github.com/ninja-build/ninja

use crate::config::NinjaUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

/// Ninja runtime implementation
#[derive(Debug, Clone, Default)]
pub struct NinjaRuntime;

impl NinjaRuntime {
    /// Create a new Ninja runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for NinjaRuntime {
    fn name(&self) -> &str {
        "ninja"
    }

    fn description(&self) -> &str {
        "Ninja - A small build system with a focus on speed"
    }

    fn aliases(&self) -> &[&str] {
        &["ninja-build"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://ninja-build.org/".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://ninja-build.org/manual.html".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/ninja-build/ninja".to_string(),
        );
        meta.insert("category".to_string(), "build-system".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // Ninja extracts directly without subdirectory
        NinjaUrlBuilder::get_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        VersionFetcherBuilder::jsdelivr("ninja-build", "ninja")
            .tool_name("ninja")
            .strip_v_prefix()
            .prerelease_markers(&["-alpha", "-beta", "-rc", "-dev"])
            .skip_prereleases()
            .limit(30)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(NinjaUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = NinjaUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Ninja executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
