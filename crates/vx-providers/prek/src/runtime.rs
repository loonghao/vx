//! prek runtime implementation
//!
//! prek is a better `pre-commit` framework, re-engineered in Rust.
//! Single binary, fully compatible with pre-commit configuration.
//!
//! Homepage: https://prek.j178.dev/
//! Repository: https://github.com/j178/prek

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

use vx_runtime::{
    Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
    layout::{ArchiveLayout, DownloadType, ExecutableLayout},
};
use vx_version_fetcher::VersionFetcherBuilder;

use crate::config::PrekUrlBuilder;

/// prek runtime implementation
#[derive(Debug, Clone, Default)]
pub struct PrekRuntime;

impl PrekRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for PrekRuntime {
    fn name(&self) -> &str {
        "prek"
    }

    fn description(&self) -> &str {
        "prek - better pre-commit, re-engineered in Rust"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://prek.j178.dev/".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/j178/prek".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://prek.j178.dev/quickstart/".to_string(),
        );
        meta.insert("category".to_string(), "devtools".to_string());
        meta.insert("license".to_string(), "MIT".to_string());
        meta
    }

    /// prek archives have a top-level directory named after the target triple
    /// (e.g., prek-x86_64-unknown-linux-gnu/prek). Use executable_layout with
    /// auto-strip (empty strip_prefix) to flatten it during installation.
    fn executable_layout(&self) -> Option<ExecutableLayout> {
        Some(ExecutableLayout {
            download_type: DownloadType::Archive,
            binary: None,
            archive: Some(ArchiveLayout {
                executable_paths: vec!["prek.exe".to_string(), "prek".to_string()],
                strip_prefix: Some(String::new()),
                permissions: None,
            }),
            msi: None,
            windows: None,
            macos: None,
            linux: None,
        })
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        platform.exe_name("prek")
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        VersionFetcherBuilder::github_releases("j178", "prek")
            .tool_name("prek")
            .strip_v_prefix()
            .skip_prereleases()
            .limit(50)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(PrekUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path(version, platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec!["prek executable not found at expected location".to_string()],
                vec!["Check download URL and extraction process".to_string()],
            )
        }
    }
}
