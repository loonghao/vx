//! # VX Download Manager
//!
//! Unified download manager for vx using turbo-cdn for optimized downloads
//! with intelligent CDN routing, geographic optimization, and multi-source fallback.

pub mod cache;
pub mod error;
pub mod manager;
pub mod monitoring;
pub mod progress;
pub mod smart_cache;
pub mod sources;
pub mod vx_config;

pub use cache::{CacheStats, DownloadCache};
pub use error::{DownloadError, Result};
pub use manager::VxDownloadManager;
pub use progress::{ProgressCallback, ProgressInfo};

// Re-export turbo-cdn types for convenience
pub use turbo_cdn::{DownloadOptions, DownloadResult, Region, Source, TurboCdn, TurboCdnConfig};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_download_manager_creation() {
        let manager = VxDownloadManager::new().await;
        assert!(manager.is_ok(), "Failed to create download manager");
    }

    #[tokio::test]
    async fn test_url_optimization() {
        let manager = VxDownloadManager::new().await.unwrap();
        let test_url = "https://github.com/rust-lang/mdBook/releases/download/v0.4.21/mdbook-v0.4.21-x86_64-unknown-linux-gnu.tar.gz";

        let optimized = manager.get_optimal_url(test_url).await;
        assert!(optimized.is_ok(), "Failed to optimize URL");
    }
}
