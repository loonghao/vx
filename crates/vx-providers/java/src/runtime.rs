//! Java runtime implementation

use crate::config::JavaUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Java runtime (Temurin JDK)
#[derive(Debug, Clone)]
pub struct JavaRuntime;

impl JavaRuntime {
    /// Create a new Java runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for JavaRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for JavaRuntime {
    fn name(&self) -> &str {
        "java"
    }

    fn description(&self) -> &str {
        "Java Development Kit (Eclipse Temurin)"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("java".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["jdk", "temurin", "openjdk"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://adoptium.net/".to_string());
        meta.insert("ecosystem".to_string(), "java".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/adoptium/temurin-build".to_string(),
        );
        meta.insert(
            "license".to_string(),
            "GPL-2.0-with-classpath-exception".to_string(),
        );
        meta
    }

    /// Java archives extract to a versioned directory like jdk-21.0.1+12
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // The actual directory name varies, so we use a pattern
        // The installer should handle finding the correct directory
        let exe_name = platform.exe_name("java");
        // On macOS, the structure is different (Contents/Home/bin/java)
        if platform.os == vx_runtime::Os::MacOS {
            format!("*/Contents/Home/bin/{}", exe_name)
        } else {
            format!("*/bin/{}", exe_name)
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch available feature versions from Adoptium API with caching
        let url = "https://api.adoptium.net/v3/info/available_releases";

        let response = ctx
            .get_cached_or_fetch("java", || async { ctx.http.get_json_value(url).await })
            .await?;

        let mut versions = Vec::new();

        // Get available LTS versions
        if let Some(lts_versions) = response
            .get("available_lts_releases")
            .and_then(|v| v.as_array())
        {
            for v in lts_versions {
                if let Some(version_num) = v.as_u64() {
                    versions.push(VersionInfo {
                        version: version_num.to_string(),
                        released_at: None,
                        prerelease: false,
                        lts: true,
                        download_url: None,
                        checksum: None,
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        // Get available non-LTS versions
        if let Some(available_versions) = response
            .get("available_releases")
            .and_then(|v| v.as_array())
        {
            for v in available_versions {
                if let Some(version_num) = v.as_u64() {
                    let version_str = version_num.to_string();
                    // Skip if already added as LTS
                    if !versions.iter().any(|vi| vi.version == version_str) {
                        versions.push(VersionInfo {
                            version: version_str,
                            released_at: None,
                            prerelease: false,
                            lts: false,
                            download_url: None,
                            checksum: None,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }

        // Sort by version number descending
        versions.sort_by(|a, b| {
            let a_num: u64 = a.version.parse().unwrap_or(0);
            let b_num: u64 = b.version.parse().unwrap_or(0);
            b_num.cmp(&a_num)
        });

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(JavaUrlBuilder::download_url(version, platform))
    }
}
