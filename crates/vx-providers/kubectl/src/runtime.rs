//! kubectl runtime implementation

use crate::config::KubectlUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

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
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "kubectl.exe"
        } else {
            "kubectl"
        };
        exe_name.to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = "https://api.github.com/repos/kubernetes/kubernetes/releases?per_page=50";

        let data = ctx
            .get_cached_or_fetch_with_url(self.name(), url, || async {
                ctx.http.get_json_value(url).await
            })
            .await?;

        let versions: Vec<VersionInfo> = data
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format from Kubernetes GitHub API"))?
            .iter()
            .filter_map(|release| {
                let tag = release.get("tag_name")?.as_str()?;
                // Remove 'v' prefix
                let version = tag.strip_prefix('v').unwrap_or(tag);
                let prerelease = release
                    .get("prerelease")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(false);

                Some(VersionInfo::new(version).with_prerelease(prerelease))
            })
            .collect();

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(KubectlUrlBuilder::download_url(version, platform))
    }
}
