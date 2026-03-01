//! GitHub and jsDelivr version fetching logic.
//!
//! Extracted from `RuntimeContext` to keep context.rs focused on dependency
//! injection wiring. All network + cache logic for GitHub Releases, GitHub Tags,
//! and the jsDelivr CDN fallback lives here.
//!
//! # Deprecation Notice
//!
//! This module is **planned for removal** in favour of
//! [`vx_version_fetcher::GitHubReleasesFetcher`], which implements the
//! unified [`vx_version_fetcher::VersionFetcher`] trait, is more composable,
//! and supports all the same options via [`vx_version_fetcher::GitHubReleasesConfig`].
//!
//! Migration guide:
//!
//! ```rust,ignore
//! // Before (deprecated):
//! use vx_runtime::GitHubReleaseOptions;
//! let versions = ctx.fetch_github_releases("helm", "helm", "helm", GitHubReleaseOptions::default()).await?;
//!
//! // After:
//! use vx_version_fetcher::{GitHubReleasesFetcher, VersionFetcher};
//! let versions = GitHubReleasesFetcher::new("helm", "helm").fetch(ctx).await?;
//! ```

use crate::traits::HttpClient;
use crate::types::VersionInfo;
use crate::version_cache::{CacheMode, CompactVersion, VersionCache};
use chrono::DateTime;
use std::collections::HashMap;
use std::sync::Arc;

/// Options for parsing GitHub releases / tags
#[allow(clippy::type_complexity)]
pub struct GitHubReleaseOptions {
    /// Number of releases to fetch per page (max 100)
    pub per_page: u32,
    /// Whether to strip 'v' prefix from tags (e.g., "v1.0.0" -> "1.0.0")
    pub strip_v_prefix: bool,
    /// Custom tag prefix to strip (takes precedence over strip_v_prefix)
    pub tag_prefix: Option<String>,
    /// Whether to skip draft releases
    pub skip_drafts: bool,
    /// Whether to skip prerelease versions
    pub skip_prereleases: bool,
    /// Custom function to detect LTS versions
    pub(crate) lts_detector: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl std::fmt::Debug for GitHubReleaseOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitHubReleaseOptions")
            .field("per_page", &self.per_page)
            .field("strip_v_prefix", &self.strip_v_prefix)
            .field("tag_prefix", &self.tag_prefix)
            .field("skip_drafts", &self.skip_drafts)
            .field("skip_prereleases", &self.skip_prereleases)
            .field("lts_detector", &self.lts_detector.is_some())
            .finish()
    }
}

impl Default for GitHubReleaseOptions {
    fn default() -> Self {
        Self {
            per_page: 50,
            strip_v_prefix: true,
            tag_prefix: None,
            skip_drafts: true,
            skip_prereleases: false,
            lts_detector: None,
        }
    }
}

impl GitHubReleaseOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn per_page(mut self, count: u32) -> Self {
        self.per_page = count.min(100);
        self
    }

    pub fn strip_v_prefix(mut self, strip: bool) -> Self {
        self.strip_v_prefix = strip;
        self
    }

    pub fn tag_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.tag_prefix = Some(prefix.into());
        self
    }

    pub fn skip_drafts(mut self, skip: bool) -> Self {
        self.skip_drafts = skip;
        self
    }

    pub fn skip_prereleases(mut self, skip: bool) -> Self {
        self.skip_prereleases = skip;
        self
    }

    pub fn lts_detector<F>(mut self, detector: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.lts_detector = Some(Box::new(detector));
        self
    }
}

/// Thin wrapper that owns the HTTP client + cache needed for GitHub fetches.
///
/// Callers obtain one via `RuntimeContext::github_fetcher()`.
pub struct GitHubFetcher<'a> {
    pub(crate) http: Arc<dyn HttpClient>,
    pub(crate) cache: Option<&'a VersionCache>,
}

impl<'a> GitHubFetcher<'a> {
    /// Fetch versions from GitHub Releases API with cache + jsDelivr fallback.
    pub async fn fetch_releases(
        &self,
        tool_name: &str,
        owner: &str,
        repo: &str,
        options: GitHubReleaseOptions,
    ) -> anyhow::Result<Vec<VersionInfo>> {
        // Fast path: version cache hit
        if let Some(cache) = self.cache {
            if let Some(versions) = cache.get(tool_name) {
                tracing::debug!(
                    "Using cached versions for {} ({} versions)",
                    tool_name,
                    versions.len()
                );
                return Ok(compact_to_version_info(versions));
            }
            if cache.mode() == CacheMode::Offline {
                return Err(anyhow::anyhow!(
                    "Offline mode: no cached versions for {}. Run without --offline to fetch.",
                    tool_name
                ));
            }
        }

        let stale = self.cache.as_ref().and_then(|c| c.get_stale(tool_name));

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases?per_page={}",
            owner, repo, options.per_page
        );
        tracing::debug!("Fetching versions for {} from {}", tool_name, url);

        match self.http.get_json_value(&url).await {
            Ok(response) => {
                if let Some(message) = response.get("message").and_then(|m| m.as_str()) {
                    if let Some(stale) = stale {
                        tracing::warn!(
                            "GitHub API error for {}: {}, using stale cache",
                            tool_name,
                            message
                        );
                        return Ok(compact_to_version_info(stale));
                    }
                    tracing::info!(
                        "GitHub API error for {}: {}, trying jsDelivr CDN...",
                        tool_name,
                        message
                    );
                    if let Ok(versions) = self
                        .try_jsdelivr_fallback(tool_name, owner, repo, &options)
                        .await
                    {
                        return Ok(versions);
                    }
                    return Err(anyhow::anyhow!(
                        "GitHub API error: {}. Set GITHUB_TOKEN or GH_TOKEN to avoid rate limits.",
                        message
                    ));
                }

                let releases = response
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("Invalid response format from GitHub API"))?;

                let compact: Vec<CompactVersion> = releases
                    .iter()
                    .filter(|r| {
                        r.get("assets")
                            .and_then(|a| a.as_array())
                            .map(|a| !a.is_empty())
                            .unwrap_or(false)
                    })
                    .filter_map(|r| parse_github_release_to_compact(r, &options))
                    .collect();

                if let Some(cache) = self.cache
                    && let Err(e) =
                        cache.set_with_options(tool_name, compact.clone(), Some(&url), None)
                {
                    tracing::warn!("Failed to cache versions for {}: {}", tool_name, e);
                }
                Ok(compact_to_version_info(compact))
            }
            Err(e) => {
                if let Some(stale) = stale {
                    tracing::warn!(
                        "Network error fetching versions for {}, using stale cache: {}",
                        tool_name,
                        e
                    );
                    return Ok(compact_to_version_info(stale));
                }
                tracing::info!(
                    "GitHub API failed for {}, trying jsDelivr CDN...",
                    tool_name
                );
                if let Ok(versions) = self
                    .try_jsdelivr_fallback(tool_name, owner, repo, &options)
                    .await
                {
                    return Ok(versions);
                }
                Err(format_network_error(e, tool_name))
            }
        }
    }

    /// Fetch versions from GitHub Tags API with cache.
    pub async fn fetch_tags(
        &self,
        tool_name: &str,
        owner: &str,
        repo: &str,
        options: GitHubReleaseOptions,
    ) -> anyhow::Result<Vec<VersionInfo>> {
        if let Some(cache) = self.cache {
            if let Some(versions) = cache.get(tool_name) {
                tracing::debug!(
                    "Using cached versions for {} ({} versions)",
                    tool_name,
                    versions.len()
                );
                return Ok(compact_to_version_info(versions));
            }
            if cache.mode() == CacheMode::Offline {
                return Err(anyhow::anyhow!(
                    "Offline mode: no cached versions for {}. Run without --offline to fetch.",
                    tool_name
                ));
            }
        }

        let stale = self.cache.as_ref().and_then(|c| c.get_stale(tool_name));

        let url = format!(
            "https://api.github.com/repos/{}/{}/tags?per_page={}",
            owner, repo, options.per_page
        );
        tracing::debug!("Fetching versions for {} from {}", tool_name, url);

        match self.http.get_json_value(&url).await {
            Ok(response) => {
                if let Some(message) = response.get("message").and_then(|m| m.as_str()) {
                    if let Some(stale) = stale {
                        tracing::warn!(
                            "GitHub API error for {}: {}, using stale cache",
                            tool_name,
                            message
                        );
                        return Ok(compact_to_version_info(stale));
                    }
                    return Err(anyhow::anyhow!(
                        "GitHub API error: {}. Set GITHUB_TOKEN or GH_TOKEN to avoid rate limits.",
                        message
                    ));
                }
                let tags = response
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("Invalid response format from GitHub API"))?;

                let compact: Vec<CompactVersion> = tags
                    .iter()
                    .filter_map(|t| parse_github_tag_to_compact(t, &options))
                    .collect();

                if let Some(cache) = self.cache
                    && let Err(e) =
                        cache.set_with_options(tool_name, compact.clone(), Some(&url), None)
                {
                    tracing::warn!("Failed to cache versions for {}: {}", tool_name, e);
                }
                Ok(compact_to_version_info(compact))
            }
            Err(e) => {
                if let Some(stale) = stale {
                    tracing::warn!(
                        "Network error fetching versions for {}, using stale cache: {}",
                        tool_name,
                        e
                    );
                    return Ok(compact_to_version_info(stale));
                }
                Err(format_network_error(e, tool_name))
            }
        }
    }
    /// jsDelivr CDN fallback — no GitHub rate limits.
    async fn try_jsdelivr_fallback(
        &self,
        tool_name: &str,
        owner: &str,
        repo: &str,
        options: &GitHubReleaseOptions,
    ) -> anyhow::Result<Vec<VersionInfo>> {
        let url = format!("https://data.jsdelivr.com/v1/package/gh/{}/{}", owner, repo);
        tracing::debug!("Fetching versions for {} from jsDelivr: {}", tool_name, url);

        let response = self.http.get_json_value(&url).await?;
        let versions_array = response
            .get("versions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid jsDelivr response format"))?;

        let mut versions: Vec<VersionInfo> = versions_array
            .iter()
            .filter_map(|v| {
                let raw = v.as_str()?;
                let version = if options.strip_v_prefix {
                    raw.trim_start_matches('v').to_string()
                } else {
                    raw.to_string()
                };
                if options.skip_prereleases {
                    let lower = version.to_lowercase();
                    if ["alpha", "beta", "-rc", "dev", "pre", "snapshot"]
                        .iter()
                        .any(|p| lower.contains(p))
                    {
                        return None;
                    }
                }
                let parts: Vec<&str> = version.split('.').collect();
                if parts.len() < 2 || parts[0].parse::<u32>().is_err() {
                    return None;
                }
                Some(VersionInfo {
                    version,
                    released_at: None,
                    lts: false,
                    prerelease: false,
                    download_url: None,
                    checksum: None,
                    metadata: HashMap::new(),
                })
            })
            .collect();

        // Newest first
        versions.sort_by(|a, b| {
            let parse = |v: &str| -> (u64, u64, u64) {
                let p: Vec<&str> = v.split('.').collect();
                (
                    p.first().and_then(|s| s.parse().ok()).unwrap_or(0),
                    p.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
                    p.get(2)
                        .and_then(|s| s.split('-').next())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                )
            };
            parse(&b.version).cmp(&parse(&a.version))
        });
        versions.truncate(options.per_page as usize);

        if versions.is_empty() {
            return Err(anyhow::anyhow!(
                "No valid versions found from jsDelivr for {}/{}",
                owner,
                repo
            ));
        }

        // Cache the result
        let compact: Vec<CompactVersion> = versions
            .iter()
            .map(|v| CompactVersion {
                version: v.version.clone(),
                prerelease: v.prerelease,
                published_at: 0,
            })
            .collect();
        if let Some(cache) = self.cache
            && let Err(e) = cache.set_with_options(tool_name, compact, Some(&url), None)
        {
            tracing::warn!("Failed to cache versions for {}: {}", tool_name, e);
        }
        Ok(versions)
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

pub(crate) fn compact_to_version_info(versions: Vec<CompactVersion>) -> Vec<VersionInfo> {
    versions
        .into_iter()
        .map(|v| {
            let released_at = if v.published_at > 0 {
                DateTime::from_timestamp(v.published_at as i64, 0)
            } else {
                None
            };
            VersionInfo {
                version: v.version,
                released_at,
                prerelease: v.prerelease,
                lts: false,
                download_url: None,
                checksum: None,
                metadata: HashMap::new(),
            }
        })
        .collect()
}

fn parse_github_release_to_compact(
    release: &serde_json::Value,
    options: &GitHubReleaseOptions,
) -> Option<CompactVersion> {
    if options.skip_drafts
        && release
            .get("draft")
            .and_then(|d| d.as_bool())
            .unwrap_or(false)
    {
        return None;
    }
    let tag = release.get("tag_name")?.as_str()?;
    let version = if let Some(prefix) = &options.tag_prefix {
        tag.strip_prefix(prefix.as_str()).unwrap_or(tag)
    } else if options.strip_v_prefix {
        tag.strip_prefix('v').unwrap_or(tag)
    } else {
        tag
    };
    let prerelease = release
        .get("prerelease")
        .and_then(|p| p.as_bool())
        .unwrap_or(false);
    if options.skip_prereleases && prerelease {
        return None;
    }
    let published_at = release
        .get("published_at")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp() as u64)
        .unwrap_or(0);
    Some(
        CompactVersion::new(version)
            .with_prerelease(prerelease)
            .with_published_at(published_at),
    )
}

fn parse_github_tag_to_compact(
    tag: &serde_json::Value,
    options: &GitHubReleaseOptions,
) -> Option<CompactVersion> {
    let tag_name = tag.get("name")?.as_str()?;
    let version = if let Some(prefix) = &options.tag_prefix {
        tag_name.strip_prefix(prefix.as_str()).unwrap_or(tag_name)
    } else if options.strip_v_prefix {
        tag_name.strip_prefix('v').unwrap_or(tag_name)
    } else {
        tag_name
    };
    let prerelease = ["alpha", "beta", "rc", "dev", "pre"]
        .iter()
        .any(|p| version.contains(p));
    if options.skip_prereleases && prerelease {
        return None;
    }
    Some(CompactVersion::new(version).with_prerelease(prerelease))
}

fn format_network_error(e: anyhow::Error, tool_name: &str) -> anyhow::Error {
    let msg = e.to_string();
    if msg.contains("timeout") || msg.contains("timed out") {
        anyhow::anyhow!(
            "Network timeout while fetching versions for {}.\n\
            Check your internet connection or set HTTPS_PROXY. Original error: {}",
            tool_name,
            msg
        )
    } else if msg.contains("rate limit") || msg.contains("403") {
        anyhow::anyhow!(
            "GitHub API rate limit exceeded for {}.\n\
            Set GITHUB_TOKEN or GH_TOKEN to increase limits. Original error: {}",
            tool_name,
            msg
        )
    } else {
        e
    }
}
