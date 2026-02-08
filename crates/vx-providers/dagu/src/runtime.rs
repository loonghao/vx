//! Dagu runtime implementation
//!
//! Dagu is a self-contained workflow engine with a built-in Web UI.
//! It uses YAML-based DAG definitions with cron scheduling support.
//!
//! Homepage: https://dagu.readthedocs.io/
//! Repository: https://github.com/dagu-org/dagu

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

use crate::config::DaguUrlBuilder;

/// Dagu runtime implementation
#[derive(Debug, Clone, Default)]
pub struct DaguRuntime;

impl DaguRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for DaguRuntime {
    fn name(&self) -> &str {
        "dagu"
    }

    fn description(&self) -> &str {
        "Dagu - self-contained workflow engine with Web UI and cron scheduling"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("devops".to_string())
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://dagu.readthedocs.io/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/dagu-org/dagu".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://dagu.readthedocs.io/".to_string(),
        );
        meta.insert("category".to_string(), "workflow".to_string());
        meta.insert("license".to_string(), "GPL-3.0".to_string());
        meta
    }

    /// Dagu archives extract directly to the binary (no subdirectory)
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        platform.exe_name("dagu")
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        VersionFetcherBuilder::github_releases("dagu-org", "dagu")
            .tool_name("dagu")
            .strip_v_prefix()
            .skip_prereleases()
            .limit(50)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(DaguUrlBuilder::download_url(version, platform))
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
                vec!["Dagu executable not found at expected location".to_string()],
                vec!["Check download URL and extraction process".to_string()],
            )
        }
    }
}
