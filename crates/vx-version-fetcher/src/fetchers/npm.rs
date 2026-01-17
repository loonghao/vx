//! npm Registry version fetcher
//!
//! Fetches version information from the npm registry API.

use crate::error::{FetchError, FetchResult};
use crate::fetcher::VersionFetcher;
use crate::utils::{version_utils, VersionInfoExt};
use async_trait::async_trait;
use vx_runtime::{RuntimeContext, VersionInfo};

/// Configuration for npm fetcher
#[derive(Debug, Clone)]
pub struct NpmConfig {
    /// Whether to skip prereleases (versions with -)
    pub skip_prereleases: bool,
    /// Maximum versions to return
    pub max_versions: usize,
    /// Whether to include release dates
    pub include_release_date: bool,
    /// LTS version pattern (versions starting with this are marked as LTS)
    pub lts_pattern: Option<String>,
}

impl Default for NpmConfig {
    fn default() -> Self {
        Self {
            skip_prereleases: true,
            max_versions: 100,
            include_release_date: true,
            lts_pattern: None,
        }
    }
}

impl NpmConfig {
    /// Set whether to skip prereleases
    pub fn with_skip_prereleases(mut self, skip: bool) -> Self {
        self.skip_prereleases = skip;
        self
    }

    /// Set maximum versions to return
    pub fn with_max_versions(mut self, max: usize) -> Self {
        self.max_versions = max;
        self
    }

    /// Set whether to include release dates
    pub fn with_include_release_date(mut self, include: bool) -> Self {
        self.include_release_date = include;
        self
    }

    /// Set LTS pattern
    pub fn with_lts_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.lts_pattern = Some(pattern.into());
        self
    }
}

/// npm Registry version fetcher
///
/// Fetches version information from `registry.npmjs.org`.
/// No rate limits, includes release dates.
///
/// # Example
///
/// ```rust,ignore
/// let fetcher = NpmFetcher::new("pnpm")
///     .with_config(NpmConfig::default()
///         .with_skip_prereleases(true)
///         .with_max_versions(100));
///
/// let versions = fetcher.fetch(ctx).await?;
/// ```
pub struct NpmFetcher {
    package: String,
    config: NpmConfig,
}

impl NpmFetcher {
    /// Create a new npm fetcher for a package
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            config: NpmConfig::default(),
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: NpmConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the API URL
    fn api_url(&self) -> String {
        format!("https://registry.npmjs.org/{}", self.package)
    }

    /// Parse a version with optional release date
    fn parse_version(
        &self,
        version: &str,
        time_obj: Option<&serde_json::Map<String, serde_json::Value>>,
    ) -> Option<VersionInfo> {
        // Skip prereleases (versions with -)
        if self.config.skip_prereleases && version.contains('-') {
            return None;
        }

        // Validate semver
        if !version_utils::is_valid_semver(version) {
            return None;
        }

        // Get release date
        let release_date = if self.config.include_release_date {
            time_obj
                .and_then(|t| t.get(version))
                .and_then(|d| d.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };

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
                .with_lts(is_lts)
                .with_optional_release_date(release_date),
        )
    }
}

#[async_trait]
impl VersionFetcher for NpmFetcher {
    async fn fetch(&self, ctx: &RuntimeContext) -> FetchResult<Vec<VersionInfo>> {
        let url = self.api_url();

        // Use caching if available
        let response = ctx
            .get_cached_or_fetch_with_url(&self.package, &url, || async {
                ctx.http.get_json_value(&url).await
            })
            .await
            .map_err(|e| FetchError::network(e.to_string()))?;

        // Parse versions object
        let versions_obj = response
            .get("versions")
            .and_then(|v| v.as_object())
            .ok_or_else(|| FetchError::invalid_format("npm", "Missing 'versions' object"))?;

        // Get time object for release dates
        let time_obj = response.get("time").and_then(|t| t.as_object());

        // Parse versions
        let mut versions: Vec<VersionInfo> = versions_obj
            .keys()
            .filter_map(|version| self.parse_version(version, time_obj))
            .collect();

        if versions.is_empty() {
            return Err(FetchError::no_versions(&self.package));
        }

        // Sort and truncate
        version_utils::sort_versions_desc(&mut versions);
        versions.truncate(self.config.max_versions);

        Ok(versions)
    }

    fn name(&self) -> &str {
        "npm"
    }

    fn source_url(&self) -> Option<String> {
        Some(format!("https://www.npmjs.com/package/{}", self.package))
    }

    fn description(&self) -> &str {
        "Fetches versions from npm registry"
    }
}
