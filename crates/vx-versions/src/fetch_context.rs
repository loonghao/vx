//! FetchContext trait - lightweight abstraction for version fetching
//!
//! This trait decouples `vx-version-fetcher` from `vx-runtime`,
//! allowing fetchers to work with any context that provides HTTP and caching.

use anyhow::Result;

/// Minimal context required by version fetchers.
///
/// `RuntimeContext` in `vx-runtime` implements this trait, so existing
/// fetchers can be called with `&ctx` as before.
#[async_trait::async_trait]
pub trait FetchContext: Send + Sync {
    /// Perform a GET request and return the response body as JSON Value
    async fn get_json_value(&self, url: &str) -> Result<serde_json::Value>;

    /// Get cached JSON data or fetch it via `get_json_value`, storing the result.
    ///
    /// - `cache_key`: key for caching (usually the tool name)
    /// - `url`: URL to fetch and store in cache metadata
    ///
    /// Default implementation: always fetches without caching.
    async fn get_cached_or_fetch(&self, cache_key: &str, url: &str) -> Result<serde_json::Value> {
        let _ = cache_key;
        self.get_json_value(url).await
    }
}
