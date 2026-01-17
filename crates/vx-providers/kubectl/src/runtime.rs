//! kubectl runtime implementation

use crate::config::KubectlUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

/// kubectl runtime
#[derive(Debug, Clone)]
pub struct KubectlRuntime;

impl KubectlRuntime {
    /// Create a new kubectl runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for KubectlRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for KubectlRuntime {
    fn name(&self) -> &str {
        "kubectl"
    }

    fn description(&self) -> &str {
        "kubectl - Kubernetes command-line tool"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("kubernetes".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["kube", "k8s"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://kubernetes.io/docs/reference/kubectl/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "kubernetes".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/kubernetes/kubectl".to_string(),
        );
        meta.insert("license".to_string(), "Apache-2.0".to_string());
        meta
    }

    /// kubectl is a single binary download (no archive)
    /// We place it under bin/ to align with generic executable lookup
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        format!("bin/{}", platform.exe_name("kubectl"))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        VersionFetcherBuilder::jsdelivr("kubernetes", "kubernetes")
            .tool_name("kubectl")
            .strip_v_prefix()
            .skip_prereleases()
            .limit(50)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(KubectlUrlBuilder::download_url(version, platform))
    }
}
