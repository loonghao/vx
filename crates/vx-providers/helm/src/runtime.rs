//! Helm runtime implementation

use crate::config::HelmUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

/// Helm runtime
#[derive(Debug, Clone)]
pub struct HelmRuntime;

impl HelmRuntime {
    /// Create a new Helm runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for HelmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for HelmRuntime {
    fn name(&self) -> &str {
        "helm"
    }

    fn description(&self) -> &str {
        "Helm - The package manager for Kubernetes"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("kubernetes".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://helm.sh/".to_string());
        meta.insert("ecosystem".to_string(), "kubernetes".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/helm/helm".to_string(),
        );
        meta.insert("license".to_string(), "Apache-2.0".to_string());
        meta
    }

    /// Helm archives extract to {os}-{arch}/helm
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        let dir_name = HelmUrlBuilder::get_archive_dir_name(platform);
        format!("{}/{}", dir_name, platform.exe_name("helm"))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        VersionFetcherBuilder::jsdelivr("helm", "helm")
            .tool_name("helm")
            .strip_v_prefix()
            .skip_prereleases()
            .limit(50)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(HelmUrlBuilder::download_url(version, platform))
    }
}
