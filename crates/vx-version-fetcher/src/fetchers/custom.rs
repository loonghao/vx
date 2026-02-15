//! Custom API version fetcher
//!
//! Allows fetching versions from any JSON API with a custom parser.

use crate::error::{FetchError, FetchResult};
use crate::fetcher::VersionFetcher;
use async_trait::async_trait;
use std::sync::Arc;
use vx_runtime::{RuntimeContext, VersionInfo};

/// Parser function type for custom API responses
pub type ParserFn =
    Arc<dyn Fn(&serde_json::Value) -> anyhow::Result<Vec<VersionInfo>> + Send + Sync>;

/// Custom API version fetcher
///
/// Allows fetching versions from any JSON API with a custom parser function.
///
/// # Example
///
/// ```rust,ignore
/// let fetcher = CustomApiFetcher::new(
///     "https://nodejs.org/dist/index.json",
///     |response| {
///         let array = response.as_array()
///             .ok_or_else(|| anyhow::anyhow!("Expected array"))?;
///
///         let versions = array.iter()
///             .filter_map(|item| {
///                 let version = item.get("version")?.as_str()?;
///                 Some(VersionInfo::new(version.trim_start_matches('v')))
///             })
///             .collect();
///
///         Ok(versions)
///     }
/// );
///
/// let versions = fetcher.fetch(ctx).await?;
/// ```
pub struct CustomApiFetcher {
    url: String,
    parser: ParserFn,
    name: String,
    cache_key: String,
}

impl CustomApiFetcher {
    /// Create a new custom API fetcher
    pub fn new<F>(url: impl Into<String>, parser: F) -> Self
    where
        F: Fn(&serde_json::Value) -> anyhow::Result<Vec<VersionInfo>> + Send + Sync + 'static,
    {
        let url = url.into();
        let cache_key = url.clone();
        Self {
            url,
            parser: Arc::new(parser),
            name: "Custom API".to_string(),
            cache_key,
        }
    }

    /// Set the fetcher name (for logging)
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the cache key
    pub fn with_cache_key(mut self, key: impl Into<String>) -> Self {
        self.cache_key = key.into();
        self
    }
}

#[async_trait]
impl VersionFetcher for CustomApiFetcher {
    async fn fetch(&self, ctx: &RuntimeContext) -> FetchResult<Vec<VersionInfo>> {
        // Use caching if available
        let response = ctx
            .get_cached_or_fetch_with_url(&self.cache_key, &self.url, || async {
                ctx.http.get_json_value(&self.url).await
            })
            .await
            .map_err(|e| FetchError::network(e.to_string()))?;

        // Parse with custom function
        let versions = (self.parser)(&response)
            .map_err(|e| FetchError::invalid_format(&self.name, format!("Parser error: {}", e)))?;

        if versions.is_empty() {
            return Err(FetchError::no_versions(&self.cache_key));
        }

        Ok(versions)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn source_url(&self) -> Option<String> {
        Some(self.url.clone())
    }

    fn description(&self) -> &str {
        "Fetches versions from a custom JSON API"
    }
}
