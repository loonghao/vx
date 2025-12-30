//! Python runtime implementation
//!
//! Uses python-build-standalone from Astral for portable Python distributions.
//! Release naming format: cpython-{python_version}+{release_date}-{platform}-{variant}.tar.gz
//!
//! Example: cpython-3.12.8+20251217-x86_64-pc-windows-msvc-shared-install_only.tar.gz
//!
//! Supports Python 3.7 to 3.12+ versions by fetching from multiple releases.

use crate::config::PythonUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Python runtime using python-build-standalone
#[derive(Debug, Clone, Default)]
pub struct PythonRuntime;

impl PythonRuntime {
    pub fn new() -> Self {
        Self
    }

    /// Parse version info from release tag and assets
    /// Returns tuples of (version, release_date) for versions available on this platform
    fn parse_versions_from_release(
        tag: &str,
        assets: &[serde_json::Value],
        platform: &Platform,
    ) -> Vec<(String, String)> {
        let mut versions = Vec::new();

        // Release tag format: YYYYMMDD (e.g., "20251217")
        let release_date = tag;

        // Pattern to match: cpython-{version}+{date}-{platform}-{variant}-install_only.tar.gz
        // Example: cpython-3.12.8+20251217-x86_64-pc-windows-msvc-shared-install_only.tar.gz
        let platform_str = PythonUrlBuilder::get_platform_string(platform);
        let variant = PythonUrlBuilder::get_variant(platform);

        // Build pattern that matches our expected filename format
        let pattern = format!(
            r"cpython-(\d+\.\d+\.\d+)\+{}-{}-{}-install_only\.(tar\.gz|tar\.zst)",
            regex::escape(release_date),
            regex::escape(platform_str),
            regex::escape(variant)
        );
        let re = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(_) => return versions,
        };

        let mut seen_versions: std::collections::HashSet<String> = std::collections::HashSet::new();

        for asset in assets {
            if let Some(name) = asset.get("name").and_then(|n| n.as_str()) {
                if let Some(caps) = re.captures(name) {
                    let version = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    if !version.is_empty() && !seen_versions.contains(version) {
                        seen_versions.insert(version.to_string());
                        versions.push((version.to_string(), release_date.to_string()));
                    }
                }
            }
        }

        versions
    }

    /// Check if a version string represents a prerelease
    fn is_prerelease(version: &str) -> bool {
        version.contains('a') || version.contains('b') || version.contains("rc")
    }

    /// Parse major.minor version from full version string
    fn parse_minor_version(version: &str) -> Option<(u32, u32)> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse().ok()?;
            let minor = parts[1].parse().ok()?;
            Some((major, minor))
        } else {
            None
        }
    }
}

#[async_trait]
impl Runtime for PythonRuntime {
    fn name(&self) -> &str {
        "python"
    }

    fn description(&self) -> &str {
        "Python programming language (3.7 - 3.12+)"
    }

    fn aliases(&self) -> &[&str] {
        &["python3", "py"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.python.org/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/astral-sh/python-build-standalone".to_string(),
        );
        meta.insert("license".to_string(), "PSF-2.0".to_string());
        meta.insert(
            "note".to_string(),
            "For pure Python development, we recommend using uv".to_string(),
        );
        meta.insert(
            "supported_versions".to_string(),
            "3.7, 3.8, 3.9, 3.10, 3.11, 3.12, 3.13+".to_string(),
        );
        meta
    }

    /// Python executable path within the extracted archive
    /// python-build-standalone extracts to: python/
    /// - Windows: python/python.exe
    /// - Unix: python/bin/python3
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        if platform.is_windows() {
            "python/python.exe".to_string()
        } else {
            "python/bin/python3".to_string()
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch more releases to cover Python 3.7 - 3.12+ versions
        // Different Python versions are available in different releases
        let url =
            "https://api.github.com/repos/astral-sh/python-build-standalone/releases?per_page=30";

        let response = ctx
            .get_cached_or_fetch("python-standalone", || async {
                ctx.http.get_json_value(url).await
            })
            .await?;

        let platform = Platform::current();

        // Map: version -> (release_date, is_prerelease)
        // Keep the newest release date for each version
        let mut version_map: HashMap<String, (String, bool)> = HashMap::new();

        if let Some(releases) = response.as_array() {
            for release in releases {
                // Skip prereleases (GitHub release prereleases, not Python prereleases)
                if release
                    .get("prerelease")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(false)
                {
                    continue;
                }

                let tag = release
                    .get("tag_name")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                // Skip non-date tags
                if tag.len() != 8 || !tag.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }

                let assets = release
                    .get("assets")
                    .and_then(|a| a.as_array())
                    .map(|a| a.as_slice())
                    .unwrap_or(&[]);

                let versions = Self::parse_versions_from_release(tag, assets, &platform);

                for (version, release_date) in versions {
                    let is_prerelease = Self::is_prerelease(&version);

                    // Only keep the newest release for each version
                    // (releases are sorted newest first from GitHub API)
                    version_map
                        .entry(version)
                        .or_insert((release_date, is_prerelease));
                }
            }
        }

        // Convert to VersionInfo list
        let mut all_versions: Vec<VersionInfo> = version_map
            .into_iter()
            .filter_map(|(version, (release_date, is_prerelease))| {
                // Filter to Python 3.7+
                if let Some((major, minor)) = Self::parse_minor_version(&version) {
                    if major == 3 && minor >= 7 {
                        return Some(
                            VersionInfo::new(&version)
                                .with_prerelease(is_prerelease)
                                .with_release_date(release_date),
                        );
                    }
                }
                None
            })
            .collect();

        // Sort by version (newest first)
        all_versions.sort_by(|a, b| {
            let v_a = semver::Version::parse(&a.version).ok();
            let v_b = semver::Version::parse(&b.version).ok();
            match (v_a, v_b) {
                (Some(a), Some(b)) => b.cmp(&a),
                _ => b.version.cmp(&a.version),
            }
        });

        Ok(all_versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // We need to find the correct release date for this version
        // First, try to fetch versions to get the mapping
        // For efficiency, we'll use a heuristic based on version

        // Try common release dates for different Python versions
        // These are the most recent releases that include each Python minor version
        let release_dates = get_release_dates_for_version(version);

        for release_date in release_dates {
            let url = PythonUrlBuilder::download_url_with_date(version, release_date, platform);
            if url.is_some() {
                return Ok(url);
            }
        }

        Ok(None)
    }
}

/// Get likely release dates for a given Python version
/// python-build-standalone releases are tagged by date (YYYYMMDD)
/// Different Python versions are available in different releases
fn get_release_dates_for_version(version: &str) -> Vec<&'static str> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        return vec!["20251217"];
    }

    let minor: u32 = parts[1].parse().unwrap_or(12);

    match minor {
        // Python 3.7 - last available in older releases
        7 => vec!["20230826", "20230726", "20230507"],
        // Python 3.8
        8 => vec!["20241206", "20241016", "20240909", "20240814", "20240726"],
        // Python 3.9
        9 => vec!["20251217", "20241206", "20241016", "20240909", "20240814"],
        // Python 3.10
        10 => vec!["20251217", "20241206", "20241016", "20240909", "20240814"],
        // Python 3.11
        11 => vec!["20251217", "20241206", "20241016", "20240909", "20240814"],
        // Python 3.12
        12 => vec!["20251217", "20241206", "20241016", "20240909", "20240814"],
        // Python 3.13+
        _ => vec!["20251217", "20241206", "20241016"],
    }
}
