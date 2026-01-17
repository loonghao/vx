//! Core VersionFetcher trait

use crate::error::FetchResult;
use async_trait::async_trait;
use vx_runtime::{RuntimeContext, VersionInfo};

/// Core trait for version fetchers
///
/// Implementations of this trait fetch version information from various sources
/// (jsDelivr, npm, PyPI, GitHub, etc.)
#[async_trait]
pub trait VersionFetcher: Send + Sync {
    /// Fetch version list from the data source
    async fn fetch(&self, ctx: &RuntimeContext) -> FetchResult<Vec<VersionInfo>>;

    /// Get the fetcher name (for debugging and logging)
    fn name(&self) -> &str;

    /// Get the data source URL (for error messages)
    fn source_url(&self) -> Option<String> {
        None
    }

    /// Get a description of this fetcher
    fn description(&self) -> &str {
        "Version fetcher"
    }
}

/// Boxed version fetcher for dynamic dispatch
pub type BoxedVersionFetcher = Box<dyn VersionFetcher>;
