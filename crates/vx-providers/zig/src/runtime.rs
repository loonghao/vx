//! Zig runtime implementation

use crate::config::ZigUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Zig runtime
#[derive(Debug, Clone)]
pub struct ZigRuntime;

impl ZigRuntime {
    /// Create a new Zig runtime
    pub fn new() -> Self {
        Self
    }

    /// Compare two version strings
    /// Returns Ordering for sorting (handles semver-like versions)
    fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
        // Extract base version (before any -dev or -rc suffix)
        fn parse_version(v: &str) -> (Vec<u32>, String) {
            let (base, suffix) = v.split_once('-').map_or((v, ""), |(b, s)| (b, s));
            let parts: Vec<u32> = base.split('.').filter_map(|p| p.parse().ok()).collect();
            (parts, suffix.to_string())
        }

        let (a_parts, a_suffix) = parse_version(a);
        let (b_parts, b_suffix) = parse_version(b);

        // Compare version numbers
        for (a_num, b_num) in a_parts.iter().zip(b_parts.iter()) {
            match a_num.cmp(b_num) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }

        // If all compared parts are equal, longer version is greater
        match a_parts.len().cmp(&b_parts.len()) {
            std::cmp::Ordering::Equal => {
                // Same base version, compare suffixes
                // No suffix > -rc > -dev
                match (a_suffix.is_empty(), b_suffix.is_empty()) {
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    _ => a_suffix.cmp(&b_suffix),
                }
            }
            other => other,
        }
    }
}

impl Default for ZigRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for ZigRuntime {
    fn name(&self) -> &str {
        "zig"
    }

    fn description(&self) -> &str {
        "Zig programming language - a systems programming language"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn aliases(&self) -> &[&str] {
        &["ziglang"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://ziglang.org/".to_string());
        meta.insert("ecosystem".to_string(), "zig".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/ziglang/zig".to_string(),
        );
        meta.insert("license".to_string(), "MIT".to_string());
        meta
    }

    /// Zig archives extract to a versioned directory
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        let dir_name = ZigUrlBuilder::get_archive_dir_name(version, platform);
        format!("{}/{}", dir_name, platform.exe_name("zig"))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from Zig download index with caching
        let url = "https://ziglang.org/download/index.json";

        let response = ctx
            .get_cached_or_fetch("zig", || async { ctx.http.get_json_value(url).await })
            .await?;

        let mut versions: Vec<VersionInfo> = response
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format from Zig API"))?
            .iter()
            .filter_map(|(version, _info)| {
                // Skip "master" and other non-version keys
                if version == "master" {
                    return None;
                }

                // Check if it's a valid semver-like version
                let is_prerelease = version.contains("-dev") || version.contains("-rc");

                Some(VersionInfo {
                    version: version.to_string(),
                    released_at: None,
                    prerelease: is_prerelease,
                    lts: false,
                    download_url: None,
                    checksum: None,
                    metadata: HashMap::new(),
                })
            })
            .collect();

        // Sort versions in descending order (newest first)
        versions.sort_by(|a, b| Self::compare_versions(&b.version, &a.version));

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(ZigUrlBuilder::download_url(version, platform))
    }
}
