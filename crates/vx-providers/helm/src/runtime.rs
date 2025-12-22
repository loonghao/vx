//! Helm runtime implementation

use crate::config::HelmUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

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
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "helm.exe"
        } else {
            "helm"
        };
        format!("{}/{}", dir_name, exe_name)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = "https://api.github.com/repos/helm/helm/releases?per_page=50";

        let data = ctx
            .get_cached_or_fetch_with_url(self.name(), url, || async {
                ctx.http.get_json_value(url).await
            })
            .await?;

        let versions: Vec<VersionInfo> = data
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format from Helm GitHub API"))?
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
        Ok(HelmUrlBuilder::download_url(version, platform))
    }
}
