//! Terraform runtime implementation

use crate::config::TerraformUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Terraform runtime
#[derive(Debug, Clone)]
pub struct TerraformRuntime;

impl TerraformRuntime {
    /// Create a new Terraform runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for TerraformRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for TerraformRuntime {
    fn name(&self) -> &str {
        "terraform"
    }

    fn description(&self) -> &str {
        "Terraform - Infrastructure as Code tool by HashiCorp"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("devops".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["tf"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.terraform.io/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "devops".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/hashicorp/terraform".to_string(),
        );
        meta.insert("license".to_string(), "BUSL-1.1".to_string());
        meta
    }

    /// Terraform archives extract directly to the binary (no subdirectory)
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        platform.exe_name("terraform")
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // HashiCorp API limit is max 20 per request
        let url = "https://api.releases.hashicorp.com/v1/releases/terraform?limit=20";

        let data = ctx
            .get_cached_or_fetch_with_url(self.name(), url, || async {
                ctx.http.get_json_value(url).await
            })
            .await?;

        let versions: Vec<VersionInfo> = data
            .as_array()
            .ok_or_else(|| {
                // Provide more context about what we received
                let type_name = if data.is_object() {
                    "object"
                } else if data.is_string() {
                    "string"
                } else if data.is_null() {
                    "null"
                } else {
                    "unknown"
                };
                anyhow::anyhow!(
                    "Invalid response format from Terraform releases API: expected array, got {}. \
                     Try clearing the cache with 'vx clean --cache'",
                    type_name
                )
            })?
            .iter()
            .filter_map(|release| {
                let version = release.get("version")?.as_str()?;
                let is_prerelease = release
                    .get("is_prerelease")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(false);

                Some(VersionInfo::new(version).with_prerelease(is_prerelease))
            })
            .collect();

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(TerraformUrlBuilder::download_url(version, platform))
    }
}
