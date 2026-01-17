//! PyPI version fetcher
//!
//! Fetches version information from the PyPI JSON API.

use crate::error::{FetchError, FetchResult};
use crate::fetcher::VersionFetcher;
use crate::utils::version_utils;
use async_trait::async_trait;
use vx_runtime::{RuntimeContext, VersionInfo};

/// Configuration for PyPI fetcher
#[derive(Debug, Clone)]
pub struct PyPiConfig {
    /// Whether to skip prereleases
    pub skip_prereleases: bool,
    /// Maximum versions to return
    pub max_versions: usize,
}

impl Default for PyPiConfig {
    fn default() -> Self {
        Self {
            skip_prereleases: true,
            max_versions: 50,
        }
    }
}

impl PyPiConfig {
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
}

/// PyPI version fetcher
///
/// Fetches version information from `pypi.org/pypi/{package}/json`.
///
/// # Example
///
/// ```rust,ignore
/// let fetcher = PyPiFetcher::new("meson")
///     .with_config(PyPiConfig::default()
///         .with_skip_prereleases(true));
///
/// let versions = fetcher.fetch(ctx).await?;
/// ```
pub struct PyPiFetcher {
    package: String,
    config: PyPiConfig,
}

impl PyPiFetcher {
    /// Create a new PyPI fetcher for a package
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            config: PyPiConfig::default(),
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: PyPiConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the API URL
    fn api_url(&self) -> String {
        format!("https://pypi.org/pypi/{}/json", self.package)
    }

    /// Check if a version is prerelease based on PyPI conventions
    fn is_prerelease_version(&self, version: &str) -> bool {
        let lower = version.to_lowercase();
        lower.contains("a")
            || lower.contains("b")
            || lower.contains("rc")
            || lower.contains("dev")
            || lower.contains("alpha")
            || lower.contains("beta")
            || lower.contains("pre")
    }

    /// Parse a version string
    fn parse_version(&self, version: &str) -> Option<VersionInfo> {
        // Skip prereleases if configured
        if self.config.skip_prereleases && self.is_prerelease_version(version) {
            return None;
        }

        // Basic validation - PyPI versions might not be strict semver
        // but should at least have numeric parts
        let first_char = version.chars().next()?;
        if !first_char.is_ascii_digit() {
            return None;
        }

        Some(VersionInfo::new(version).with_prerelease(false))
    }
}

#[async_trait]
impl VersionFetcher for PyPiFetcher {
    async fn fetch(&self, ctx: &RuntimeContext) -> FetchResult<Vec<VersionInfo>> {
        let url = self.api_url();

        // Use caching if available
        let response = ctx
            .get_cached_or_fetch_with_url(&self.package, &url, || async {
                ctx.http.get_json_value(&url).await
            })
            .await
            .map_err(|e| FetchError::network(e.to_string()))?;

        // Parse releases object (keys are version strings)
        let releases = response
            .get("releases")
            .and_then(|r| r.as_object())
            .ok_or_else(|| FetchError::invalid_format("PyPI", "Missing 'releases' object"))?;

        // Parse versions
        let mut versions: Vec<VersionInfo> = releases
            .keys()
            .filter_map(|version| self.parse_version(version))
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
        "PyPI"
    }

    fn source_url(&self) -> Option<String> {
        Some(format!("https://pypi.org/project/{}/", self.package))
    }

    fn description(&self) -> &str {
        "Fetches versions from PyPI (Python Package Index)"
    }
}
