//! GitHub Releases version fetcher
//!
//! Fetches version information from GitHub Releases API with jsDelivr fallback.

use crate::error::{FetchError, FetchResult};
use crate::fetcher::VersionFetcher;
use crate::fetchers::JsDelivrFetcher;
use crate::utils::version_utils;
use async_trait::async_trait;
use vx_runtime::{RuntimeContext, VersionInfo};

/// Configuration for GitHub Releases fetcher
#[derive(Debug, Clone)]
pub struct GitHubReleasesConfig {
    /// Whether to strip 'v' prefix from version tags
    pub strip_v_prefix: bool,
    /// Custom tag prefix to strip (e.g., "jq-", "bun-v")
    pub tag_prefix: Option<String>,
    /// Whether to skip prereleases
    pub skip_prereleases: bool,
    /// Number of releases per page
    pub per_page: usize,
    /// LTS version pattern
    pub lts_pattern: Option<String>,
    /// Whether to fallback to jsDelivr on GitHub API failure
    pub jsdelivr_fallback: bool,
}

impl Default for GitHubReleasesConfig {
    fn default() -> Self {
        Self {
            strip_v_prefix: true,
            tag_prefix: None,
            skip_prereleases: true,
            per_page: 30,
            lts_pattern: None,
            jsdelivr_fallback: true,
        }
    }
}

impl GitHubReleasesConfig {
    /// Set whether to strip 'v' prefix
    pub fn with_strip_v_prefix(mut self, strip: bool) -> Self {
        self.strip_v_prefix = strip;
        self
    }

    /// Set custom tag prefix
    pub fn with_tag_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.tag_prefix = Some(prefix.into());
        self
    }

    /// Set whether to skip prereleases
    pub fn with_skip_prereleases(mut self, skip: bool) -> Self {
        self.skip_prereleases = skip;
        self
    }

    /// Set releases per page
    pub fn with_per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page;
        self
    }

    /// Set LTS pattern
    pub fn with_lts_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.lts_pattern = Some(pattern.into());
        self
    }

    /// Set whether to fallback to jsDelivr
    pub fn with_jsdelivr_fallback(mut self, fallback: bool) -> Self {
        self.jsdelivr_fallback = fallback;
        self
    }
}

/// GitHub Releases version fetcher
///
/// Fetches version information from GitHub Releases API.
/// Supports automatic fallback to jsDelivr when rate limited.
///
/// # Example
///
/// ```rust,ignore
/// let fetcher = GitHubReleasesFetcher::new("helm", "helm")
///     .with_config(GitHubReleasesConfig::default()
///         .with_strip_v_prefix(true)
///         .with_skip_prereleases(true));
///
/// let versions = fetcher.fetch(ctx).await?;
/// ```
pub struct GitHubReleasesFetcher {
    owner: String,
    repo: String,
    tool_name: String,
    config: GitHubReleasesConfig,
}

impl GitHubReleasesFetcher {
    /// Create a new GitHub Releases fetcher
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        let owner = owner.into();
        let repo = repo.into();
        let tool_name = repo.clone();
        Self {
            owner,
            repo,
            tool_name,
            config: GitHubReleasesConfig::default(),
        }
    }

    /// Set the tool name (for logging/caching)
    pub fn with_tool_name(mut self, name: impl Into<String>) -> Self {
        self.tool_name = name.into();
        self
    }

    /// Set configuration
    pub fn with_config(mut self, config: GitHubReleasesConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the API URL
    fn api_url(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/releases?per_page={}",
            self.owner, self.repo, self.config.per_page
        )
    }

    /// Parse a version from tag name
    fn parse_version(&self, tag_name: &str, is_prerelease: bool) -> Option<VersionInfo> {
        // Skip GitHub prereleases if configured
        if self.config.skip_prereleases && is_prerelease {
            return None;
        }

        // Strip prefix
        let version = if let Some(ref prefix) = self.config.tag_prefix {
            tag_name.strip_prefix(prefix)?
        } else if self.config.strip_v_prefix {
            tag_name.trim_start_matches('v')
        } else {
            tag_name
        };

        // Skip prereleases based on version string
        if self.config.skip_prereleases && version_utils::is_prerelease(version) {
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
                .with_prerelease(is_prerelease)
                .with_lts(is_lts),
        )
    }

    /// Fetch versions from GitHub API
    async fn fetch_from_github(&self, ctx: &RuntimeContext) -> FetchResult<Vec<VersionInfo>> {
        let url = self.api_url();

        let response = ctx
            .http
            .get_json_value(&url)
            .await
            .map_err(|e| FetchError::network(e.to_string()))?;

        // Parse releases array
        let releases = response
            .as_array()
            .ok_or_else(|| FetchError::invalid_format("GitHub", "Expected array of releases"))?;

        // Parse versions
        let mut versions: Vec<VersionInfo> = releases
            .iter()
            .filter_map(|release| {
                let tag_name = release.get("tag_name")?.as_str()?;
                let is_prerelease = release
                    .get("prerelease")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(false);
                self.parse_version(tag_name, is_prerelease)
            })
            .collect();

        if versions.is_empty() {
            return Err(FetchError::no_versions(&self.tool_name));
        }

        // Sort
        version_utils::sort_versions_desc(&mut versions);

        Ok(versions)
    }

    /// Create jsDelivr fallback fetcher
    fn create_jsdelivr_fallback(&self) -> JsDelivrFetcher {
        use crate::fetchers::jsdelivr::JsDelivrConfig;

        let config = JsDelivrConfig::default()
            .with_skip_prereleases(self.config.skip_prereleases)
            .with_max_versions(self.config.per_page);

        let config = if let Some(ref prefix) = self.config.tag_prefix {
            config.with_strip_prefix(prefix)
        } else if self.config.strip_v_prefix {
            config.with_strip_prefix("v")
        } else {
            config
        };

        let config = if let Some(ref lts) = self.config.lts_pattern {
            config.with_lts_pattern(lts)
        } else {
            config
        };

        JsDelivrFetcher::new(&self.owner, &self.repo)
            .with_tool_name(&self.tool_name)
            .with_config(config)
    }
}

#[async_trait]
impl VersionFetcher for GitHubReleasesFetcher {
    async fn fetch(&self, ctx: &RuntimeContext) -> FetchResult<Vec<VersionInfo>> {
        // Try GitHub API first
        match self.fetch_from_github(ctx).await {
            Ok(versions) => Ok(versions),
            Err(e) if self.config.jsdelivr_fallback => {
                tracing::warn!(
                    "GitHub API failed for {}, falling back to jsDelivr: {}",
                    self.tool_name,
                    e
                );
                self.create_jsdelivr_fallback().fetch(ctx).await
            }
            Err(e) => Err(e),
        }
    }

    fn name(&self) -> &str {
        "GitHub Releases"
    }

    fn source_url(&self) -> Option<String> {
        Some(format!(
            "https://github.com/{}/{}/releases",
            self.owner, self.repo
        ))
    }

    fn description(&self) -> &str {
        "Fetches versions from GitHub Releases API with jsDelivr fallback"
    }
}
