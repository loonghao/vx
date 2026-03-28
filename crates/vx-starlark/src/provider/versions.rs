//! Version fetching logic for Starlark providers.
//!
//! Handles all version descriptor resolution and JSON transform strategies.
//!
//! # Caching
//!
//! `execute_fetch_versions` uses a two-level cache:
//! - **L1 (memory)**: per-process, instant lookup, keyed by provider name + script hash
//! - **L2 (disk)**: `~/.vx/cache/versions/<name>.json`, TTL 24h, survives restarts
//!
//! Cache invalidation:
//! - TTL expiry (default 24h)
//! - Script content change (new hash → automatic miss)

use crate::context::{ProviderContext, VersionInfo};
use crate::engine::StarlarkEngine;
use crate::error::{Error, Result};
use tracing::{debug, info, warn};
use vx_version_fetcher::VersionFetcherBuilder;

use super::StarlarkProvider;
use super::version_cache::global_version_cache;

impl StarlarkProvider {
    /// Execute fetch_versions function using the Starlark engine.
    ///
    /// **Cache-aware**: checks L1 (memory) → L2 (disk) before executing Starlark.
    /// On a cache miss, executes the Starlark function and stores the result.
    ///
    /// Handles two return shapes from Starlark:
    ///
    /// 1. **Descriptor dict** (`__type == "github_versions"`): returned by
    ///    `releases_to_versions(github_releases(...))` in http.star.
    ///    The Rust layer resolves this by calling the GitHub API directly,
    ///    keeping Starlark pure (no real HTTP in scripts).
    ///
    /// 2. **Plain list** of `{version, lts, prerelease, date}` dicts:
    ///    returned by custom `fetch_versions` implementations that build
    ///    the list themselves.
    pub(super) async fn execute_fetch_versions(
        &self,
        ctx: &ProviderContext,
    ) -> Result<Vec<VersionInfo>> {
        let provider_name = &self.meta.name;
        let hash_hex = self.script_hash_hex();
        let cache = global_version_cache();

        // For multi-runtime providers (e.g. build-tools with just/cmake/ninja),
        // the cache key must include the runtime name so that each runtime gets
        // its own cache entry. Without this, "just" versions (1.x) would be
        // returned for "cmake" queries (which should return 3.x/4.x).
        let cache_key = match ctx.runtime_name.as_deref() {
            Some(rt) if !rt.is_empty() => format!("{}/{}", provider_name, rt),
            _ => provider_name.clone(),
        };

        // ── Cache lookup (L1 → L2) ────────────────────────────────────────────
        if let Some(cached) = cache.get(&cache_key, &hash_hex).await {
            debug!(
                provider = %provider_name,
                cache_key = %cache_key,
                count = %cached.len(),
                "fetch_versions: returning cached versions"
            );
            return Ok(cached);
        }

        debug!(
            provider = %provider_name,
            cache_key = %cache_key,
            "fetch_versions: cache miss, executing Starlark"
        );

        // ── Execute Starlark ──────────────────────────────────────────────────
        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "fetch_versions",
            ctx,
            &[],
        );

        let versions = match result {
            Ok(json) => {
                // Shape 1: github_versions descriptor from http.star
                if let Some(type_str) = json.get("__type").and_then(|t| t.as_str())
                    && type_str == "github_versions"
                {
                    self.resolve_github_versions_descriptor(&json).await?
                }
                // Shape 2: unified fetch_json_versions descriptor (replaces go_versions etc.)
                else if let Some(type_str) = json.get("__type").and_then(|t| t.as_str())
                    && type_str == "fetch_json_versions"
                {
                    self.resolve_fetch_json_versions_descriptor(&json).await?
                }
                // Shape 3: legacy go_versions descriptor (kept for backward compat)
                else if let Some(type_str) = json.get("__type").and_then(|t| t.as_str())
                    && type_str == "go_versions"
                {
                    self.resolve_go_versions_descriptor(&json).await?
                }
                // Shape 4: plain list of version dicts
                else if let Some(arr) = json.as_array() {
                    arr.iter()
                        .filter_map(|v| {
                            let version = v.get("version")?.as_str()?.to_string();
                            Some(VersionInfo {
                                version,
                                lts: v.get("lts").and_then(|l| l.as_bool()).unwrap_or(false),
                                stable: v.get("stable").and_then(|s| s.as_bool()).unwrap_or(true),
                                date: v
                                    .get("date")
                                    .and_then(|d| d.as_str())
                                    .map(|s| s.to_string()),
                            })
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                warn!(
                    provider = %self.meta.name,
                    "fetch_versions() not found in provider script"
                );
                return Ok(vec![]);
            }
            Err(e) => return Err(e),
        };

        // ── Cache write (L1 + L2) ─────────────────────────────────────────────
        if !versions.is_empty() {
            info!(
                provider = %provider_name,
                cache_key = %cache_key,
                count = %versions.len(),
                "fetch_versions: caching {} versions",
                versions.len()
            );
            cache.put(&cache_key, &hash_hex, &versions).await;
        }

        Ok(versions)
    }

    // ── Descriptor resolvers ──────────────────────────────────────────────────

    /// Resolve a `github_versions` descriptor by calling the GitHub API.
    ///
    /// Delegates to `vx-version-fetcher`'s `GitHubReleasesFetcher`, which handles:
    /// - GITHUB_TOKEN / GH_TOKEN authentication
    /// - Exponential backoff retry on transient errors
    /// - Automatic jsDelivr CDN fallback on rate limit
    ///
    /// The descriptor shape (produced by `releases_to_versions(github_releases(...))` in http.star):
    /// ```json
    /// {
    ///   "__type":           "github_versions",
    ///   "source": {
    ///     "__type":             "github_releases",
    ///     "owner":              "jj-vcs",
    ///     "repo":               "jj",
    ///     "include_prereleases": false,
    ///     "url":                "https://api.github.com/repos/jj-vcs/jj/releases?per_page=50"
    ///   },
    ///   "strip_v_prefix":   true,
    ///   "skip_prereleases": true
    /// }
    /// ```
    async fn resolve_github_versions_descriptor(
        &self,
        descriptor: &serde_json::Value,
    ) -> Result<Vec<VersionInfo>> {
        let source = descriptor.get("source").ok_or_else(|| {
            Error::EvalError("github_versions descriptor missing 'source'".into())
        })?;

        let owner = source.get("owner").and_then(|o| o.as_str()).unwrap_or("");
        let repo = source.get("repo").and_then(|r| r.as_str()).unwrap_or("");

        let skip_prereleases = descriptor
            .get("skip_prereleases")
            .and_then(|s| s.as_bool())
            .unwrap_or(true);

        let strip_v = descriptor
            .get("strip_v_prefix")
            .and_then(|s| s.as_bool())
            .unwrap_or(true);

        // Optional custom tag prefix (e.g. "bun-v" for bun's "bun-v1.2.3" tags)
        let tag_prefix = descriptor
            .get("tag_prefix")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string());

        debug!(
            provider = %self.meta.name,
            owner = %owner,
            repo = %repo,
            "Resolving github_versions descriptor via vx-version-fetcher"
        );

        // Build fetcher using vx-version-fetcher (handles retry, token, jsDelivr fallback)
        let mut builder =
            VersionFetcherBuilder::github_releases(owner, repo).tool_name(&self.meta.name);

        if skip_prereleases {
            builder = builder.skip_prereleases();
        } else {
            builder = builder.include_prereleases();
        }

        // tag_prefix takes priority over strip_v_prefix
        if let Some(prefix) = tag_prefix {
            builder = builder.tag_prefix(prefix);
        } else if strip_v {
            builder = builder.strip_v_prefix();
        }

        let fetcher = builder.build();
        let ctx = build_minimal_runtime_ctx();
        let runtime_versions = fetcher
            .fetch(&ctx)
            .await
            .map_err(|e| Error::EvalError(format!("GitHub API fetch failed: {}", e)))?;

        let versions: Vec<VersionInfo> = runtime_versions
            .into_iter()
            .map(|v| VersionInfo {
                version: v.version,
                lts: v.lts,
                stable: !v.prerelease,
                date: v.released_at.map(|dt| dt.to_rfc3339()),
            })
            .collect();

        debug!(
            provider = %self.meta.name,
            count = versions.len(),
            "Resolved {} versions from GitHub API",
            versions.len()
        );

        Ok(versions)
    }

    /// Resolve a `fetch_json_versions` descriptor — the unified JSON API version fetcher.
    ///
    /// This is the single generic resolver that handles all non-GitHub JSON APIs.
    /// The `transform` field selects the parsing strategy for the raw JSON response.
    ///
    /// Descriptor shape (produced by `fetch_json_versions()` in http.star):
    /// ```json
    /// {
    ///   "__type":    "fetch_json_versions",
    ///   "url":       "https://nodejs.org/dist/index.json",
    ///   "transform": "nodejs_org",
    ///   "headers":   {}
    /// }
    /// ```
    ///
    /// Supported transform strategies:
    /// - `"go_versions"`        — go.dev API: `[{"version": "go1.21.0", "stable": true}]`
    /// - `"nodejs_org"`         — nodejs.org: `[{"version": "v20.0.0", "lts": "Iron"}]`
    /// - `"pypi"`               — PyPI JSON: `{"info": {"version": "..."}, "releases": {...}}`
    /// - `"npm_registry"`       — npm registry: `{"versions": {"1.0.0": {...}}}`
    /// - `"hashicorp_releases"` — HashiCorp: `{"versions": {"1.0.0": {"status": "supported"}}}`
    /// - `"adoptium"`           — Eclipse Adoptium Java API
    /// - `"github_tags"`        — GitHub tags API
    /// - `"vscode_releases"`    — VS Code update API
    /// - `"gcloud_manifest"`    — Google Cloud SDK manifest
    /// - `"dotnet_releases"`    — .NET releases index
    async fn resolve_fetch_json_versions_descriptor(
        &self,
        descriptor: &serde_json::Value,
    ) -> Result<Vec<VersionInfo>> {
        let url = descriptor
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| {
                Error::EvalError("fetch_json_versions descriptor missing 'url'".into())
            })?;

        let transform = descriptor
            .get("transform")
            .and_then(|t| t.as_str())
            .unwrap_or("generic");

        debug!(
            provider = %self.meta.name,
            url = %url,
            transform = %transform,
            "Resolving fetch_json_versions descriptor via HTTP"
        );

        // Special case: python_build_standalone uses GitHub releases API.
        // Use GitHubReleasesFetcher (with retry + jsDelivr fallback) to fetch
        // the raw releases JSON, then apply our custom transform.
        if transform == "python_build_standalone" {
            return self.resolve_python_build_standalone_versions(url).await;
        }

        // Build a custom API fetcher using vx-version-fetcher
        // The transform function is passed as the parser to CustomApiFetcher
        let url_owned = url.to_string();
        let transform_owned = transform.to_string();
        let provider_name = self.meta.name.clone();

        let fetcher =
            VersionFetcherBuilder::custom_api(url_owned.clone(), move |raw: &serde_json::Value| {
                let versions = match transform_owned.as_str() {
                    "go_versions" => Self::transform_go_versions(raw)?,
                    "nodejs_org" => Self::transform_nodejs_org(raw)?,
                    "pypi" => Self::transform_pypi(raw)?,
                    "npm_registry" => Self::transform_npm_registry(raw)?,
                    "hashicorp_releases" => Self::transform_hashicorp_releases(raw)?,
                    "adoptium" => Self::transform_adoptium(raw)?,
                    "github_tags" => Self::transform_github_tags(raw)?,
                    "vscode_releases" => Self::transform_vscode_releases(raw)?,
                    "gcloud_manifest" => Self::transform_gcloud_manifest(raw)?,
                    "dotnet_releases" => Self::transform_dotnet_releases(raw)?,
                    "python_build_standalone" => Self::transform_python_build_standalone(raw)?,
                    other => {
                        tracing::warn!(
                            transform = %other,
                            "Unknown fetch_json_versions transform, returning empty list"
                        );
                        vec![]
                    }
                };
                // Convert crate::context::VersionInfo -> vx_runtime::VersionInfo
                Ok(versions
                    .into_iter()
                    .map(|v| vx_runtime::VersionInfo {
                        version: v.version,
                        lts: v.lts,
                        prerelease: !v.stable,
                        released_at: v.date.as_deref().and_then(|d| {
                            chrono::DateTime::parse_from_rfc3339(d)
                                .ok()
                                .map(|dt| dt.with_timezone(&chrono::Utc))
                        }),
                        download_url: None,
                        checksum: None,
                        metadata: std::collections::HashMap::new(),
                    })
                    .collect())
            })
            .with_name(format!("fetch_json_versions({})", transform))
            .build();

        let ctx = build_minimal_runtime_ctx();
        let runtime_versions = fetcher
            .fetch(&ctx)
            .await
            .map_err(|e| Error::EvalError(format!("HTTP API fetch failed for {}: {}", url, e)))?;

        let versions: Vec<VersionInfo> = runtime_versions
            .into_iter()
            .map(|v| VersionInfo {
                version: v.version,
                lts: v.lts,
                stable: !v.prerelease,
                date: v.released_at.map(|dt| dt.to_rfc3339()),
            })
            .collect();

        debug!(
            provider = %provider_name,
            count = versions.len(),
            transform = %transform,
            "Resolved {} versions via fetch_json_versions",
            versions.len()
        );

        Ok(versions)
    }

    /// Resolve python-build-standalone versions by fetching GitHub releases with pagination.
    ///
    /// Uses small page sizes (per_page=15) to avoid GitHub API timeouts that occur
    /// with large responses (per_page=50 reliably returns 504 for this repo).
    /// Fetches up to 20 pages (300 releases) to cover all Python versions back to 3.7.
    ///
    /// The `astral-sh/python-build-standalone` repo releases frequently (multiple
    /// per month as of 2025+), so we need enough pages to reach older Python versions
    /// like 3.7.x and 3.8.x whose last builds were in 2023.
    async fn resolve_python_build_standalone_versions(
        &self,
        url: &str,
    ) -> Result<Vec<VersionInfo>> {
        debug!(
            provider = %self.meta.name,
            url = %url,
            "Resolving python-build-standalone versions via GitHub API (paginated)"
        );

        // Try fetching from GitHub API first, fall back to well-known versions on failure.
        match self.fetch_python_versions_from_github(url).await {
            Ok(versions) if !versions.is_empty() => {
                // Merge: real versions take priority, add any missing well-known versions
                let merged = Self::merge_with_wellknown_python_versions(versions);
                debug!(
                    provider = %self.meta.name,
                    count = merged.len(),
                    "Resolved {} Python versions (GitHub API + well-known fallback)",
                    merged.len()
                );
                Ok(merged)
            }
            Ok(_empty) => {
                warn!(
                    provider = %self.meta.name,
                    "GitHub API returned no Python versions, using well-known fallback"
                );
                Ok(Self::wellknown_python_versions())
            }
            Err(e) => {
                warn!(
                    provider = %self.meta.name,
                    error = %e,
                    "GitHub API failed for python-build-standalone, using well-known fallback"
                );
                Ok(Self::wellknown_python_versions())
            }
        }
    }

    /// Fetch Python versions from GitHub API with pagination.
    /// Returns an error if all pages fail (network/rate limit issues).
    async fn fetch_python_versions_from_github(&self, url: &str) -> Result<Vec<VersionInfo>> {
        let base_url = if let Some(pos) = url.find('?') {
            &url[..pos]
        } else {
            url
        };

        let client = StarlarkHttpClient::new();
        let mut all_releases: Vec<serde_json::Value> = Vec::new();

        for page in 1..=20u32 {
            let page_url = format!("{}?per_page=15&page={}", base_url, page);
            debug!(
                provider = %self.meta.name,
                page = page,
                "Fetching python-build-standalone releases page {}", page
            );

            let raw = client.fetch_json(&page_url).await.map_err(|e| {
                Error::EvalError(format!(
                    "GitHub API fetch failed for python-build-standalone (page {}): {}",
                    page, e
                ))
            })?;

            if let Some(message) = raw.get("message").and_then(|m| m.as_str()) {
                return Err(Error::EvalError(format!("GitHub API error: {}", message)));
            }

            let page_releases = raw.as_array().ok_or_else(|| {
                Error::EvalError("python_build_standalone: expected JSON array".into())
            })?;

            if page_releases.is_empty() {
                break;
            }

            all_releases.extend(page_releases.iter().cloned());

            let distinct_versions = Self::count_distinct_python_versions(&all_releases);
            if distinct_versions >= 12 {
                debug!(
                    provider = %self.meta.name,
                    distinct_versions = distinct_versions,
                    "Found enough Python versions (>= 12 distinct), stopping pagination"
                );
                break;
            }
        }

        let combined = serde_json::Value::Array(all_releases);
        Self::transform_python_build_standalone(&combined)
    }

    /// Well-known Python versions from python-build-standalone.
    ///
    /// These are hardcoded as a reliable fallback when the GitHub API is unavailable
    /// (rate limiting, network issues, timeouts). The `date` field contains the
    /// python-build-standalone release tag (build tag), required by `download_url`
    /// to construct the asset URL via `ctx.version_date`.
    ///
    /// These should be updated periodically when new Python patch versions are released.
    /// The build tag (`date`) must correspond to an actual python-build-standalone release
    /// that contains the specified cpython version.
    fn wellknown_python_versions() -> Vec<VersionInfo> {
        // Last updated: 2026-03-28 (build tag: 20260325)
        let versions = [
            // Python 3.13.x (current)
            ("3.13.4", "20260325", false),
            ("3.13.3", "20250317", false),
            ("3.13.2", "20250212", false),
            ("3.13.1", "20250115", false),
            ("3.13.0", "20241016", false),
            // Python 3.12.x (LTS - even minor)
            ("3.12.11", "20260325", true),
            ("3.12.10", "20250317", true),
            ("3.12.9", "20250212", true),
            ("3.12.8", "20250115", true),
            ("3.12.7", "20241016", true),
            // Python 3.11.x
            ("3.11.13", "20260325", false),
            ("3.11.12", "20250317", false),
            ("3.11.11", "20250115", false),
            ("3.11.10", "20241016", false),
            // Python 3.10.x (LTS - even minor)
            ("3.10.20", "20260325", true),
            ("3.10.17", "20250317", true),
            ("3.10.16", "20250115", true),
            ("3.10.15", "20241016", true),
            // Python 3.9.x
            ("3.9.22", "20250317", false),
            ("3.9.21", "20250115", false),
            ("3.9.20", "20241016", false),
            // Python 3.8.x (LTS - even minor, EOL but still available)
            ("3.8.20", "20241016", true),
            // Note: Python 3.7.x is NOT available in python-build-standalone.
            // The project never included 3.7 builds. Minimum supported version is 3.8.
        ];

        let mut result: Vec<VersionInfo> = versions
            .iter()
            .map(|(version, build_tag, lts)| VersionInfo {
                version: version.to_string(),
                lts: *lts,
                stable: true,
                date: Some(build_tag.to_string()),
            })
            .collect();

        result.sort_by(|a, b| {
            let parse =
                |v: &str| -> Vec<u64> { v.split('.').filter_map(|p| p.parse().ok()).collect() };
            parse(&b.version).cmp(&parse(&a.version))
        });

        result
    }

    /// Merge real-time versions with well-known fallback versions.
    ///
    /// Real-time versions take priority. Well-known versions fill in any gaps
    /// (e.g., older Python versions like 3.8/3.9 that may not appear in recent
    /// GitHub releases pages).
    fn merge_with_wellknown_python_versions(
        mut real_versions: Vec<VersionInfo>,
    ) -> Vec<VersionInfo> {
        let existing: std::collections::HashSet<String> =
            real_versions.iter().map(|v| v.version.clone()).collect();

        for fallback in Self::wellknown_python_versions() {
            if !existing.contains(&fallback.version) {
                real_versions.push(fallback);
            }
        }

        real_versions.sort_by(|a, b| {
            let parse =
                |v: &str| -> Vec<u64> { v.split('.').filter_map(|p| p.parse().ok()).collect() };
            parse(&b.version).cmp(&parse(&a.version))
        });

        real_versions
    }

    /// Count distinct Python versions found in a list of releases.
    fn count_distinct_python_versions(releases: &[serde_json::Value]) -> usize {
        let mut seen = std::collections::HashSet::new();
        for release in releases {
            let assets = release
                .get("assets")
                .and_then(|a| a.as_array())
                .map(|a| a.as_slice())
                .unwrap_or(&[]);
            for asset in assets {
                let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
                if !name.starts_with("cpython-") || !name.contains("install_only") {
                    continue;
                }
                let after = &name["cpython-".len()..];
                if let Some(plus) = after.find('+') {
                    let ver = &after[..plus];
                    if !ver.contains('a') && !ver.contains('b') && !ver.contains("rc") {
                        // Only track major.minor (e.g. "3.12")
                        let minor_key: String =
                            ver.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
                        seen.insert(minor_key);
                    }
                }
            }
        }
        seen.len()
    }

    /// Resolve a `go_versions` descriptor by calling the go.dev API.
    ///
    /// **Deprecated**: Use `fetch_json_versions` with `transform = "go_versions"` instead.
    /// Kept for backward compatibility with existing provider.star files.
    async fn resolve_go_versions_descriptor(
        &self,
        descriptor: &serde_json::Value,
    ) -> Result<Vec<VersionInfo>> {
        let url = descriptor
            .get("url")
            .and_then(|u| u.as_str())
            .unwrap_or("https://go.dev/dl/?mode=json&include=all")
            .to_string();

        debug!(
            provider = %self.meta.name,
            url = %url,
            "Resolving go_versions descriptor via vx-version-fetcher"
        );

        let fetcher = VersionFetcherBuilder::custom_api(url.clone(), |raw: &serde_json::Value| {
            let versions = Self::transform_go_versions(raw)?;
            Ok(versions
                .into_iter()
                .map(|v| vx_runtime::VersionInfo {
                    version: v.version,
                    lts: v.lts,
                    prerelease: !v.stable,
                    released_at: None,
                    download_url: None,
                    checksum: None,
                    metadata: std::collections::HashMap::new(),
                })
                .collect())
        })
        .with_name("go.dev API")
        .build();

        let ctx = build_minimal_runtime_ctx();
        let runtime_versions = fetcher
            .fetch(&ctx)
            .await
            .map_err(|e| Error::EvalError(format!("go.dev API fetch failed: {}", e)))?;

        let versions: Vec<VersionInfo> = runtime_versions
            .into_iter()
            .map(|v| VersionInfo {
                version: v.version,
                lts: v.lts,
                stable: !v.prerelease,
                date: None,
            })
            .collect();

        debug!(
            provider = %self.meta.name,
            count = versions.len(),
            "Resolved {} versions from go.dev API",
            versions.len()
        );

        Ok(versions)
    }

    // ── Transform strategies ──────────────────────────────────────────────────

    /// Transform go.dev API response: `[{"version": "go1.21.0", "stable": true}]`
    fn transform_go_versions(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        let releases = raw
            .as_array()
            .ok_or_else(|| Error::EvalError("go_versions: expected JSON array".into()))?;

        let mut versions = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for release in releases {
            let v = release
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let v = v.strip_prefix("go").unwrap_or(v);
            if v.is_empty() || seen.contains(v) {
                continue;
            }
            seen.insert(v.to_string());
            let stable = release
                .get("stable")
                .and_then(|s| s.as_bool())
                .unwrap_or(false);
            versions.push(VersionInfo {
                version: v.to_string(),
                lts: stable,
                stable,
                date: None,
            });
        }
        Ok(versions)
    }

    /// Transform nodejs.org API response: `[{"version": "v20.0.0", "lts": "Iron"|false}]`
    fn transform_nodejs_org(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        let releases = raw
            .as_array()
            .ok_or_else(|| Error::EvalError("nodejs_org: expected JSON array".into()))?;

        let versions = releases
            .iter()
            .filter_map(|r| {
                let tag = r.get("version")?.as_str()?;
                let version = tag.strip_prefix('v').unwrap_or(tag).to_string();
                if version.is_empty() {
                    return None;
                }
                // lts is either a string (LTS codename) or false
                let lts = r
                    .get("lts")
                    .map(|l| !l.is_boolean() || l.as_bool() == Some(true))
                    .unwrap_or(false);
                let date = r
                    .get("date")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());
                Some(VersionInfo {
                    version,
                    lts,
                    stable: true,
                    date,
                })
            })
            .collect();
        Ok(versions)
    }

    /// Transform PyPI JSON API: `{"info": {"version": "latest"}, "releases": {"1.0.0": [...]}}`
    fn transform_pypi(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        let releases = raw
            .get("releases")
            .and_then(|r| r.as_object())
            .ok_or_else(|| Error::EvalError("pypi: expected 'releases' object".into()))?;

        let mut versions: Vec<VersionInfo> = releases
            .keys()
            .filter(|v| !v.contains('a') && !v.contains('b') && !v.contains("rc"))
            .map(|v| VersionInfo {
                version: v.clone(),
                lts: false,
                stable: true,
                date: None,
            })
            .collect();

        versions.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(versions)
    }

    /// Transform npm registry response: `{"versions": {"1.0.0": {...}, ...}}`
    fn transform_npm_registry(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        let versions_obj = raw
            .get("versions")
            .and_then(|v| v.as_object())
            .ok_or_else(|| Error::EvalError("npm_registry: expected 'versions' object".into()))?;

        let mut versions: Vec<VersionInfo> = versions_obj
            .keys()
            .filter(|v| !v.contains('-')) // skip pre-releases like "1.0.0-beta.1"
            .map(|v| VersionInfo {
                version: v.clone(),
                lts: false,
                stable: true,
                date: None,
            })
            .collect();

        versions.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(versions)
    }

    /// Transform HashiCorp releases API: `{"versions": {"1.0.0": {"status": "supported"}}}`
    fn transform_hashicorp_releases(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        let versions_obj = raw
            .get("versions")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                Error::EvalError("hashicorp_releases: expected 'versions' object".into())
            })?;

        let mut versions: Vec<VersionInfo> = versions_obj
            .iter()
            .filter(|(v, _)| !v.contains('-'))
            .map(|(v, info)| {
                let status = info.get("status").and_then(|s| s.as_str()).unwrap_or("");
                VersionInfo {
                    version: v.clone(),
                    lts: status == "supported",
                    stable: status != "deprecated",
                    date: None,
                }
            })
            .collect();

        versions.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(versions)
    }

    /// Transform Eclipse Adoptium API response for Java versions
    fn transform_adoptium(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        // Adoptium returns: {"available_releases": [8, 11, 17, 21], "most_recent_lts": 21}
        let available = raw
            .get("available_releases")
            .and_then(|a| a.as_array())
            .ok_or_else(|| {
                Error::EvalError("adoptium: expected 'available_releases' array".into())
            })?;

        let most_recent_lts = raw
            .get("most_recent_lts")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let versions = available
            .iter()
            .filter_map(|v| v.as_u64())
            .map(|major| VersionInfo {
                version: major.to_string(),
                lts: major == most_recent_lts || major % 4 == 1, // LTS: 8, 11, 17, 21...
                stable: true,
                date: None,
            })
            .collect();

        Ok(versions)
    }

    /// Transform GitHub tags API: `[{"name": "v1.0.0", "commit": {...}}]`
    fn transform_github_tags(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        let tags = raw
            .as_array()
            .ok_or_else(|| Error::EvalError("github_tags: expected JSON array".into()))?;

        let versions = tags
            .iter()
            .filter_map(|t| {
                let tag = t.get("name")?.as_str()?;
                let version = tag.strip_prefix('v').unwrap_or(tag).to_string();
                if version.is_empty() {
                    return None;
                }
                Some(VersionInfo {
                    version,
                    lts: false,
                    stable: true,
                    date: None,
                })
            })
            .collect();

        Ok(versions)
    }

    /// Transform VS Code update API: `["1.85.0", "1.84.2", ...]` (plain string array)
    fn transform_vscode_releases(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        let releases = raw
            .as_array()
            .ok_or_else(|| Error::EvalError("vscode_releases: expected JSON array".into()))?;

        let versions = releases
            .iter()
            .filter_map(|v| {
                let version = v.as_str()?.to_string();
                if version.is_empty() {
                    return None;
                }
                Some(VersionInfo {
                    version,
                    lts: true,
                    stable: true,
                    date: None,
                })
            })
            .collect();

        Ok(versions)
    }

    /// Transform Google Cloud SDK manifest: `{"version": "456.0.0", ...}`
    ///
    /// The gcloud manifest only exposes the current/latest version.
    fn transform_gcloud_manifest(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        let version = raw
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if version.is_empty() {
            return Ok(vec![]);
        }

        Ok(vec![VersionInfo {
            version,
            lts: true,
            stable: true,
            date: None,
        }])
    }

    /// Transform .NET releases index: two-level API (index → channel releases)
    ///
    /// Since `fetch_json_versions` only supports a single URL, we use the index
    /// URL and extract only the channel-level version info (not individual SDK versions).
    fn transform_dotnet_releases(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        // The index returns: {"releases-index": [{"channel-version": "8.0", "latest-sdk": "8.0.100", ...}]}
        let index = raw
            .get("releases-index")
            .and_then(|i| i.as_array())
            .ok_or_else(|| {
                Error::EvalError("dotnet_releases: expected 'releases-index' array".into())
            })?;

        let mut versions: Vec<VersionInfo> = index
            .iter()
            .filter(|channel| {
                channel
                    .get("support-phase")
                    .and_then(|s| s.as_str())
                    .map(|s| s != "eol")
                    .unwrap_or(true)
            })
            .filter_map(|channel| {
                let latest_sdk = channel.get("latest-sdk")?.as_str()?.to_string();
                if latest_sdk.is_empty() {
                    return None;
                }
                let is_lts = channel
                    .get("release-type")
                    .and_then(|t| t.as_str())
                    .map(|t| t == "lts")
                    .unwrap_or(false);
                Some(VersionInfo {
                    version: latest_sdk,
                    lts: is_lts,
                    stable: true,
                    date: None,
                })
            })
            .collect();

        versions.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(versions)
    }

    /// Transform python-build-standalone GitHub releases API.
    ///
    /// python-build-standalone releases are tagged by date (e.g. `20240107`).
    /// Each release contains assets named like:
    ///   `cpython-3.12.1+20240107-x86_64-pc-windows-msvc-install_only_stripped.tar.gz`
    ///
    /// This transform:
    /// 1. Iterates over all releases (each is a date-tagged release)
    /// 2. For each release, scans asset names for `cpython-{version}+{tag}-...-install_only`
    /// 3. Extracts the Python version (e.g. `3.12.1`) and the build tag (e.g. `20240107`)
    /// 4. Stores the build tag in the `date` field so `download_url` can reconstruct the URL
    /// 5. De-duplicates: keeps only the latest build tag per Python version
    fn transform_python_build_standalone(raw: &serde_json::Value) -> Result<Vec<VersionInfo>> {
        // Handle GitHub API error responses (e.g. rate limit exceeded)
        if let Some(message) = raw.get("message").and_then(|m| m.as_str()) {
            return Err(Error::EvalError(format!("GitHub API error: {}", message)));
        }

        let releases = raw.as_array().ok_or_else(|| {
            Error::EvalError("python_build_standalone: expected JSON array".into())
        })?;

        // Map: python_version -> (build_tag, lts)
        // We keep the first (most recent) occurrence of each Python version.
        let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for release in releases {
            let tag = release
                .get("tag_name")
                .and_then(|t| t.as_str())
                .unwrap_or("");

            let assets = release
                .get("assets")
                .and_then(|a| a.as_array())
                .map(|a| a.as_slice())
                .unwrap_or(&[]);

            for asset in assets {
                let asset_name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");

                // Match: cpython-{version}+{tag}-...-install_only...
                // We only care about install_only_stripped or install_only archives.
                if !asset_name.starts_with("cpython-") {
                    continue;
                }
                if !asset_name.contains("install_only") {
                    continue;
                }
                // Skip debug builds
                if asset_name.contains("-debug-") {
                    continue;
                }

                // Extract Python version: between "cpython-" and "+"
                // e.g. "cpython-3.12.1+20240107-..." → "3.12.1"
                let after_cpython = &asset_name["cpython-".len()..];
                let py_version = if let Some(plus_pos) = after_cpython.find('+') {
                    &after_cpython[..plus_pos]
                } else {
                    continue;
                };

                // Skip pre-release versions (e.g. "3.13.0a1")
                if py_version.contains('a') || py_version.contains('b') || py_version.contains("rc")
                {
                    continue;
                }

                // Only record the first (most recent release) occurrence
                if !seen.contains_key(py_version) {
                    seen.insert(py_version.to_string(), tag.to_string());
                }
            }
        }

        let mut versions: Vec<VersionInfo> = seen
            .into_iter()
            .map(|(py_version, build_tag)| {
                // LTS heuristic: Python 3.x where x is even (3.8, 3.10, 3.12...)
                let lts = py_version
                    .split('.')
                    .nth(1)
                    .and_then(|minor| minor.parse::<u64>().ok())
                    .map(|minor| minor % 2 == 0)
                    .unwrap_or(false);

                VersionInfo {
                    version: py_version,
                    lts,
                    stable: true,
                    // Store build_tag in date field — used by download_url via version_date
                    date: Some(build_tag),
                }
            })
            .collect();

        // Sort by version descending (semver-ish)
        versions.sort_by(|a, b| {
            let parse =
                |v: &str| -> Vec<u64> { v.split('.').filter_map(|p| p.parse().ok()).collect() };
            parse(&b.version).cmp(&parse(&a.version))
        });

        Ok(versions)
    }
}

/// Build a minimal `RuntimeContext` for use in descriptor resolvers.
///
/// This context only has a real HTTP client; all other fields use lightweight
/// no-op implementations. It is used to drive `vx-version-fetcher` fetchers
/// without pulling in the full `vx-runtime-http` dependency.
fn build_minimal_runtime_ctx() -> vx_runtime::RuntimeContext {
    use std::sync::Arc;
    use vx_runtime::{MockFileSystem, MockInstaller, MockPathProvider, RuntimeContext};

    let http = Arc::new(StarlarkHttpClient::new());
    let fs = Arc::new(MockFileSystem::new());
    let paths = Arc::new(MockPathProvider::new("/tmp/vx-starlark"));
    let installer = Arc::new(MockInstaller::new());
    RuntimeContext::new(paths, http, fs, installer)
}

/// Lightweight reqwest-based `HttpClient` implementation for Starlark descriptor resolvers.
///
/// Only `get_json_value` is used by `vx-version-fetcher` fetchers.
struct StarlarkHttpClient {
    client: reqwest::Client,
}

impl StarlarkHttpClient {
    fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("vx/0.1 (https://github.com/loonghao/vx)")
            .timeout(std::time::Duration::from_secs(30))
            // Force HTTP/1.1 — some GitHub API endpoints return 503 with HTTP/2
            .http1_only()
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// Fetch a URL and return the response body as a JSON Value.
    /// Adds GitHub token if available and the URL is a GitHub API endpoint.
    /// Retries up to 3 times on 5xx errors.
    async fn fetch_json(&self, url: &str) -> anyhow::Result<serde_json::Value> {
        let mut last_err = anyhow::anyhow!("No attempts made");
        for attempt in 0..3u32 {
            if attempt > 0 {
                // Exponential backoff: 1s, 2s
                tokio::time::sleep(std::time::Duration::from_secs(attempt as u64)).await;
                debug!(
                    "Retrying GitHub API request (attempt {}): {}",
                    attempt + 1,
                    url
                );
            }
            let mut req = self
                .client
                .get(url)
                .header("Accept", "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28");
            if url.contains("api.github.com")
                && let Ok(token) =
                    std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("GH_TOKEN"))
            {
                req = req.bearer_auth(token);
            }
            match req.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return Ok(response.json().await?);
                    }
                    let body = response.text().await.unwrap_or_default();
                    last_err = anyhow::anyhow!("HTTP {} from {}: {}", status, url, body);
                    // Only retry on 5xx errors
                    if !status.is_server_error() {
                        return Err(last_err);
                    }
                }
                Err(e) => {
                    last_err = anyhow::anyhow!("Request failed: {}", e);
                }
            }
        }
        Err(last_err)
    }
}

#[async_trait::async_trait]
impl vx_runtime::HttpClient for StarlarkHttpClient {
    async fn get(&self, url: &str) -> anyhow::Result<String> {
        let response = self.client.get(url).send().await?;
        Ok(response.text().await?)
    }

    async fn get_json_value(&self, url: &str) -> anyhow::Result<serde_json::Value> {
        // Support GITHUB_TOKEN for GitHub API requests
        let mut req = self
            .client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        if url.contains("api.github.com")
            && let Ok(token) = std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("GH_TOKEN"))
        {
            req = req.bearer_auth(token);
        }
        let response = req.send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {} from {}: {}", status, url, body));
        }
        Ok(response.json().await?)
    }

    async fn download(&self, url: &str, dest: &std::path::Path) -> anyhow::Result<()> {
        let bytes = self.client.get(url).send().await?.bytes().await?;
        std::fs::write(dest, bytes)?;
        Ok(())
    }

    async fn download_with_progress(
        &self,
        url: &str,
        dest: &std::path::Path,
        _on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    ) -> anyhow::Result<()> {
        self.download(url, dest).await
    }
}
