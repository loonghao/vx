//! Version Fetcher Builder
//!
//! Provides a fluent API for creating version fetchers.

use crate::error::FetchResult;
use crate::fetcher::{BoxedVersionFetcher, VersionFetcher};
use crate::fetchers::{
    CustomApiFetcher, GitHubReleasesConfig, GitHubReleasesFetcher, JsDelivrConfig, JsDelivrFetcher,
    NpmConfig, NpmFetcher, PyPiConfig, PyPiFetcher,
};
use async_trait::async_trait;
use std::sync::Arc;
use vx_runtime::{RuntimeContext, VersionInfo};

/// Type alias for custom version parser function
type VersionParser = dyn Fn(&serde_json::Value) -> anyhow::Result<Vec<VersionInfo>> + Send + Sync;

/// Builder for creating version fetchers
///
/// Provides a fluent API for configuring and creating version fetchers.
///
/// # Examples
///
/// ```rust,ignore
/// // jsDelivr (GitHub proxy) - most common, no rate limits
/// let fetcher = VersionFetcherBuilder::jsdelivr("helm", "helm")
///     .strip_prefix("v")
///     .skip_prereleases()
///     .limit(50)
///     .build();
///
/// // npm registry
/// let fetcher = VersionFetcherBuilder::npm("pnpm")
///     .skip_prereleases()
///     .lts_pattern("1.22.")
///     .build();
///
/// // GitHub releases with jsDelivr fallback
/// let fetcher = VersionFetcherBuilder::github_releases("owner", "repo")
///     .strip_v_prefix()
///     .build();
///
/// // Custom API
/// let fetcher = VersionFetcherBuilder::custom_api("https://api.example.com/versions", |response| {
///     // parse response
///     Ok(vec![])
/// })
/// .build();
/// ```
pub struct VersionFetcherBuilder {
    inner: BuilderInner,
}

enum BuilderInner {
    JsDelivr {
        owner: String,
        repo: String,
        tool_name: Option<String>,
        config: JsDelivrConfig,
    },
    Npm {
        package: String,
        config: NpmConfig,
    },
    PyPi {
        package: String,
        config: PyPiConfig,
    },
    GitHub {
        owner: String,
        repo: String,
        tool_name: Option<String>,
        config: GitHubReleasesConfig,
    },
    Custom {
        url: String,
        parser: Arc<VersionParser>,
        name: Option<String>,
        cache_key: Option<String>,
    },
    Static {
        versions: Vec<VersionInfo>,
    },
}

impl VersionFetcherBuilder {
    // ==================== Constructors ====================

    /// Create a jsDelivr CDN fetcher (GitHub proxy)
    ///
    /// This is the recommended default for most GitHub-hosted tools.
    /// No rate limits, fast CDN, always available.
    ///
    /// # Example
    /// ```rust,ignore
    /// let fetcher = VersionFetcherBuilder::jsdelivr("helm", "helm")
    ///     .strip_prefix("v")
    ///     .skip_prereleases()
    ///     .build();
    /// ```
    pub fn jsdelivr(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            inner: BuilderInner::JsDelivr {
                owner: owner.into(),
                repo: repo.into(),
                tool_name: None,
                config: JsDelivrConfig::default(),
            },
        }
    }

    /// Create an npm registry fetcher
    ///
    /// For npm packages. No rate limits, includes release dates.
    ///
    /// # Example
    /// ```rust,ignore
    /// let fetcher = VersionFetcherBuilder::npm("pnpm")
    ///     .skip_prereleases()
    ///     .build();
    /// ```
    pub fn npm(package: impl Into<String>) -> Self {
        Self {
            inner: BuilderInner::Npm {
                package: package.into(),
                config: NpmConfig::default(),
            },
        }
    }

    /// Create a PyPI fetcher
    ///
    /// For Python packages on PyPI.
    ///
    /// # Example
    /// ```rust,ignore
    /// let fetcher = VersionFetcherBuilder::pypi("meson")
    ///     .skip_prereleases()
    ///     .build();
    /// ```
    pub fn pypi(package: impl Into<String>) -> Self {
        Self {
            inner: BuilderInner::PyPi {
                package: package.into(),
                config: PyPiConfig::default(),
            },
        }
    }

    /// Create a GitHub Releases fetcher
    ///
    /// Uses GitHub API directly. Has rate limits but includes richer release info.
    /// Automatically falls back to jsDelivr if rate limited.
    ///
    /// # Example
    /// ```rust,ignore
    /// let fetcher = VersionFetcherBuilder::github_releases("helm", "helm")
    ///     .strip_v_prefix()
    ///     .skip_prereleases()
    ///     .build();
    /// ```
    pub fn github_releases(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            inner: BuilderInner::GitHub {
                owner: owner.into(),
                repo: repo.into(),
                tool_name: None,
                config: GitHubReleasesConfig::default(),
            },
        }
    }

    /// Create a custom API fetcher
    ///
    /// For any JSON API with a custom parser function.
    ///
    /// # Example
    /// ```rust,ignore
    /// let fetcher = VersionFetcherBuilder::custom_api(
    ///     "https://nodejs.org/dist/index.json",
    ///     |response| {
    ///         // Custom parsing logic
    ///         Ok(vec![])
    ///     }
    /// )
    /// .with_name("Node.js Official")
    /// .build();
    /// ```
    pub fn custom_api<F>(url: impl Into<String>, parser: F) -> Self
    where
        F: Fn(&serde_json::Value) -> anyhow::Result<Vec<VersionInfo>> + Send + Sync + 'static,
    {
        Self {
            inner: BuilderInner::Custom {
                url: url.into(),
                parser: Arc::new(parser),
                name: None,
                cache_key: None,
            },
        }
    }

    /// Create a static version fetcher
    ///
    /// Returns a predefined list of versions. Useful for tools with known versions.
    ///
    /// # Example
    /// ```rust,ignore
    /// let fetcher = VersionFetcherBuilder::static_versions(vec!["1.0.0", "2.0.0"])
    ///     .build();
    /// ```
    pub fn static_versions(versions: Vec<impl Into<String>>) -> Self {
        Self {
            inner: BuilderInner::Static {
                versions: versions
                    .into_iter()
                    .map(|v| VersionInfo::new(v.into()))
                    .collect(),
            },
        }
    }

    // ==================== Common Configuration ====================

    /// Set the tool name (for caching and logging)
    pub fn tool_name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        match &mut self.inner {
            BuilderInner::JsDelivr { tool_name, .. } => *tool_name = Some(name),
            BuilderInner::GitHub { tool_name, .. } => *tool_name = Some(name),
            BuilderInner::Custom { cache_key, .. } => *cache_key = Some(name),
            _ => {}
        }
        self
    }

    /// Strip 'v' prefix from version tags
    pub fn strip_v_prefix(self) -> Self {
        self.strip_prefix("v")
    }

    /// Strip custom prefix from version tags
    ///
    /// # Example
    /// ```rust,ignore
    /// // For tags like "jq-1.7" -> "1.7"
    /// .strip_prefix("jq-")
    ///
    /// // For tags like "bun-v1.0" -> "1.0"
    /// .strip_prefix("bun-v")
    /// ```
    pub fn strip_prefix(mut self, prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();
        match &mut self.inner {
            BuilderInner::JsDelivr { config, .. } => {
                config.strip_prefix = Some(prefix);
            }
            BuilderInner::GitHub { config, .. } => {
                if prefix == "v" {
                    config.strip_v_prefix = true;
                } else {
                    config.tag_prefix = Some(prefix);
                }
            }
            _ => {}
        }
        self
    }

    /// Set tag prefix (alias for strip_prefix)
    ///
    /// More explicit name for complex prefixes like "bun-v" or "jq-"
    pub fn tag_prefix(self, prefix: impl Into<String>) -> Self {
        self.strip_prefix(prefix)
    }

    /// Skip prerelease versions
    pub fn skip_prereleases(mut self) -> Self {
        match &mut self.inner {
            BuilderInner::JsDelivr { config, .. } => config.skip_prereleases = true,
            BuilderInner::Npm { config, .. } => config.skip_prereleases = true,
            BuilderInner::PyPi { config, .. } => config.skip_prereleases = true,
            BuilderInner::GitHub { config, .. } => config.skip_prereleases = true,
            _ => {}
        }
        self
    }

    /// Include prerelease versions
    pub fn include_prereleases(mut self) -> Self {
        match &mut self.inner {
            BuilderInner::JsDelivr { config, .. } => config.skip_prereleases = false,
            BuilderInner::Npm { config, .. } => config.skip_prereleases = false,
            BuilderInner::PyPi { config, .. } => config.skip_prereleases = false,
            BuilderInner::GitHub { config, .. } => config.skip_prereleases = false,
            _ => {}
        }
        self
    }

    /// Set custom prerelease markers
    ///
    /// # Example
    /// ```rust,ignore
    /// .prerelease_markers(&["canary", "-alpha", "-beta", "-rc"])
    /// ```
    pub fn prerelease_markers(mut self, markers: &[&str]) -> Self {
        let markers: Vec<String> = markers.iter().map(|s| s.to_string()).collect();
        if let BuilderInner::JsDelivr { config, .. } = &mut self.inner {
            config.prerelease_markers = markers;
        }
        self
    }

    /// Limit number of versions returned
    pub fn limit(mut self, max: usize) -> Self {
        match &mut self.inner {
            BuilderInner::JsDelivr { config, .. } => config.max_versions = max,
            BuilderInner::Npm { config, .. } => config.max_versions = max,
            BuilderInner::PyPi { config, .. } => config.max_versions = max,
            BuilderInner::GitHub { config, .. } => config.per_page = max,
            _ => {}
        }
        self
    }

    /// Set LTS pattern (versions starting with this pattern are marked as LTS)
    ///
    /// # Example
    /// ```rust,ignore
    /// // Mark all 1.22.x versions as LTS
    /// .lts_pattern("1.22.")
    /// ```
    pub fn lts_pattern(mut self, pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        match &mut self.inner {
            BuilderInner::JsDelivr { config, .. } => config.lts_pattern = Some(pattern),
            BuilderInner::Npm { config, .. } => config.lts_pattern = Some(pattern),
            BuilderInner::GitHub { config, .. } => config.lts_pattern = Some(pattern),
            _ => {}
        }
        self
    }

    /// Set custom fetcher name (for logging)
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        if let BuilderInner::Custom {
            name: custom_name, ..
        } = &mut self.inner
        {
            *custom_name = Some(name);
        }
        self
    }

    /// Disable jsDelivr fallback for GitHub fetcher
    pub fn no_jsdelivr_fallback(mut self) -> Self {
        if let BuilderInner::GitHub { config, .. } = &mut self.inner {
            config.jsdelivr_fallback = false;
        }
        self
    }

    // ==================== Build ====================

    /// Build the version fetcher
    pub fn build(self) -> BoxedVersionFetcher {
        match self.inner {
            BuilderInner::JsDelivr {
                owner,
                repo,
                tool_name,
                config,
            } => {
                let mut fetcher = JsDelivrFetcher::new(owner, repo).with_config(config);
                if let Some(name) = tool_name {
                    fetcher = fetcher.with_tool_name(name);
                }
                Box::new(fetcher)
            }
            BuilderInner::Npm { package, config } => {
                Box::new(NpmFetcher::new(package).with_config(config))
            }
            BuilderInner::PyPi { package, config } => {
                Box::new(PyPiFetcher::new(package).with_config(config))
            }
            BuilderInner::GitHub {
                owner,
                repo,
                tool_name,
                config,
            } => {
                let mut fetcher = GitHubReleasesFetcher::new(owner, repo).with_config(config);
                if let Some(name) = tool_name {
                    fetcher = fetcher.with_tool_name(name);
                }
                Box::new(fetcher)
            }
            BuilderInner::Custom {
                url,
                parser,
                name,
                cache_key,
            } => {
                let mut fetcher = CustomApiFetcher::new(url.clone(), move |v| (parser.clone())(v));
                if let Some(n) = name {
                    fetcher = fetcher.with_name(n);
                }
                if let Some(k) = cache_key {
                    fetcher = fetcher.with_cache_key(k);
                }
                Box::new(fetcher)
            }
            BuilderInner::Static { versions } => Box::new(StaticVersionFetcher { versions }),
        }
    }
}

/// Static version fetcher (returns predefined versions)
struct StaticVersionFetcher {
    versions: Vec<VersionInfo>,
}

#[async_trait]
impl VersionFetcher for StaticVersionFetcher {
    async fn fetch(&self, _ctx: &RuntimeContext) -> FetchResult<Vec<VersionInfo>> {
        Ok(self.versions.clone())
    }

    fn name(&self) -> &str {
        "Static"
    }

    fn description(&self) -> &str {
        "Returns predefined version list"
    }
}
