//! jsDelivr CDN version fetcher
//!
//! Uses jsDelivr's API to fetch GitHub repository tag lists,
//! avoiding GitHub API rate limit issues.

use crate::error::{FetchError, FetchResult};
use crate::fetcher::VersionFetcher;
use crate::utils::version_utils;
use async_trait::async_trait;
use vx_runtime::{RuntimeContext, VersionInfo};

/// Configuration for jsDelivr fetcher
#[derive(Debug, Clone)]
pub struct JsDelivrConfig {
    /// Version prefix to strip (e.g., "v", "jq-", "bun-v")
    pub strip_prefix: Option<String>,
    /// Whether to skip prereleases
    pub skip_prereleases: bool,
    /// Custom prerelease markers (if empty, use defaults)
    pub prerelease_markers: Vec<String>,
    /// Maximum versions to return
    pub max_versions: usize,
    /// LTS version detector
    pub lts_pattern: Option<String>,
}

impl Default for JsDelivrConfig {
    fn default() -> Self {
        Self {
            strip_prefix: None,
            skip_prereleases: true,
            prerelease_markers: vec![],
            max_versions: 50,
            lts_pattern: None,
        }
    }
}

impl JsDelivrConfig {
    /// Create a new config with strip_prefix
    pub fn with_strip_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.strip_prefix = Some(prefix.into());
        self
    }

    /// Set whether to skip prereleases
    pub fn with_skip_prereleases(mut self, skip: bool) -> Self {
        self.skip_prereleases = skip;
        self
    }

    /// Set custom prerelease markers
    pub fn with_prerelease_markers(mut self, markers: Vec<String>) -> Self {
        self.prerelease_markers = markers;
        self
    }

    /// Set maximum versions to return
    pub fn with_max_versions(mut self, max: usize) -> Self {
        self.max_versions = max;
        self
    }

    /// Set LTS pattern (versions matching this pattern are marked as LTS)
    pub fn with_lts_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.lts_pattern = Some(pattern.into());
        self
    }
}

/// jsDelivr CDN version fetcher
///
/// Uses jsDelivr's API (`data.jsdelivr.com`) to fetch version lists from GitHub
/// repositories without GitHub API rate limits.
///
/// # Example
///
/// ```rust,ignore
/// let fetcher = JsDelivrFetcher::new("helm", "helm")
///     .with_config(JsDelivrConfig::default()
///         .with_strip_prefix("v")
///         .with_skip_prereleases(true));
///
/// let versions = fetcher.fetch(ctx).await?;
/// ```
pub struct JsDelivrFetcher {
    owner: String,
    repo: String,
    tool_name: String,
    config: JsDelivrConfig,
}

impl JsDelivrFetcher {
    /// Create a new jsDelivr fetcher for a GitHub repository
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        let owner = owner.into();
        let repo = repo.into();
        let tool_name = repo.clone();
        Self {
            owner,
            repo,
            tool_name,
            config: JsDelivrConfig::default(),
        }
    }

    /// Set the tool name (for logging/caching)
    pub fn with_tool_name(mut self, name: impl Into<String>) -> Self {
        self.tool_name = name.into();
        self
    }

    /// Set configuration
    pub fn with_config(mut self, config: JsDelivrConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the API URL
    fn api_url(&self) -> String {
        format!(
            "https://data.jsdelivr.com/v1/package/gh/{}/{}",
            self.owner, self.repo
        )
    }

    /// Parse a version string according to config
    fn parse_version(&self, version_str: &str) -> Option<VersionInfo> {
        // Strip prefix if present, otherwise use version as-is
        // Note: jsDelivr often returns versions without 'v' prefix even if GitHub tags have it
        let version = match &self.config.strip_prefix {
            Some(prefix) => version_str
                .strip_prefix(prefix)
                .unwrap_or_else(|| version_str.trim_start_matches('v')),
            None => version_str.trim_start_matches('v'),
        };

        // Check prerelease
        let markers: Vec<&str> = if self.config.prerelease_markers.is_empty() {
            version_utils::DEFAULT_PRERELEASE_MARKERS.to_vec()
        } else {
            self.config
                .prerelease_markers
                .iter()
                .map(|s| s.as_str())
                .collect()
        };

        if self.config.skip_prereleases
            && version_utils::is_prerelease_with_markers(version, &markers)
        {
            return None;
        }

        // Validate semver
        if !version_utils::is_valid_semver(version) {
            return None;
        }

        // Check LTS
        let is_lts = self
            .config
            .lts_pattern
            .as_ref()
            .map(|pattern| version.starts_with(pattern))
            .unwrap_or(false);

        Some(
            VersionInfo::new(version)
                .with_prerelease(false)
                .with_lts(is_lts),
        )
    }
}

#[async_trait]
impl VersionFetcher for JsDelivrFetcher {
    async fn fetch(&self, ctx: &RuntimeContext) -> FetchResult<Vec<VersionInfo>> {
        let url = self.api_url();

        // Use caching if available
        let response = ctx
            .get_cached_or_fetch_with_url(&self.tool_name, &url, || async {
                ctx.http.get_json_value(&url).await
            })
            .await
            .map_err(|e| FetchError::network(e.to_string()))?;

        // Parse response
        let versions_array = response
            .get("versions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| FetchError::invalid_format("jsDelivr", "Missing 'versions' array"))?;

        // Parse versions
        let mut versions: Vec<VersionInfo> = versions_array
            .iter()
            .filter_map(|v| v.as_str())
            .filter_map(|v| self.parse_version(v))
            .collect();

        if versions.is_empty() {
            return Err(FetchError::no_versions(&self.tool_name));
        }

        // Sort and truncate
        version_utils::sort_versions_desc(&mut versions);
        versions.truncate(self.config.max_versions);

        Ok(versions)
    }

    fn name(&self) -> &str {
        "jsDelivr"
    }

    fn source_url(&self) -> Option<String> {
        Some(format!("https://github.com/{}/{}", self.owner, self.repo))
    }

    fn description(&self) -> &str {
        "Fetches versions from jsDelivr CDN (GitHub proxy)"
    }
}
