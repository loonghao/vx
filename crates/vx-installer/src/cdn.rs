//! CDN acceleration module for vx-installer
//!
//! This module provides optional CDN acceleration using turbo-cdn.
//! When the `cdn-acceleration` feature is enabled, downloads will be
//! automatically optimized using the best available CDN mirrors.

use crate::Result;

/// CDN optimizer for download URLs
#[derive(Debug, Clone)]
pub struct CdnOptimizer {
    enabled: bool,
}

impl CdnOptimizer {
    /// Create a new CDN optimizer
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Check if CDN acceleration is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Optimize a download URL using CDN mirrors
    ///
    /// When CDN acceleration is enabled and the `cdn-acceleration` feature is active,
    /// this will return an optimized URL from the best available CDN mirror.
    /// Otherwise, it returns the original URL.
    pub async fn optimize_url(&self, url: &str) -> Result<String> {
        if !self.enabled {
            return Ok(url.to_string());
        }

        #[cfg(feature = "cdn-acceleration")]
        {
            match turbo_cdn::async_api::quick::optimize_url(url).await {
                Ok(optimized) => {
                    tracing::debug!(
                        original = url,
                        optimized = %optimized,
                        "CDN URL optimized"
                    );
                    Ok(optimized)
                }
                Err(e) => {
                    tracing::warn!(
                        url = url,
                        error = %e,
                        "CDN optimization failed, using original URL"
                    );
                    Ok(url.to_string())
                }
            }
        }

        #[cfg(not(feature = "cdn-acceleration"))]
        {
            Ok(url.to_string())
        }
    }
}

impl Default for CdnOptimizer {
    fn default() -> Self {
        // CDN acceleration is disabled by default
        Self::new(false)
    }
}

/// Configuration for CDN acceleration
#[derive(Debug, Clone, Default)]
pub struct CdnConfig {
    /// Whether CDN acceleration is enabled
    pub enabled: bool,
    /// Preferred region (auto-detected if not set)
    pub region: Option<String>,
}

impl CdnConfig {
    /// Create a new CDN configuration with acceleration enabled
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            region: None,
        }
    }

    /// Create a new CDN configuration with acceleration disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            region: None,
        }
    }

    /// Set the preferred region
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdn_optimizer_disabled() {
        let optimizer = CdnOptimizer::new(false);
        assert!(!optimizer.is_enabled());
    }

    #[test]
    fn test_cdn_optimizer_enabled() {
        let optimizer = CdnOptimizer::new(true);
        assert!(optimizer.is_enabled());
    }

    #[test]
    fn test_cdn_config_default() {
        let config = CdnConfig::default();
        assert!(!config.enabled);
        assert!(config.region.is_none());
    }

    #[test]
    fn test_cdn_config_enabled() {
        let config = CdnConfig::enabled();
        assert!(config.enabled);
    }

    #[test]
    fn test_cdn_config_with_region() {
        let config = CdnConfig::enabled().with_region("china");
        assert!(config.enabled);
        assert_eq!(config.region, Some("china".to_string()));
    }

    #[tokio::test]
    async fn test_optimize_url_when_disabled() {
        let optimizer = CdnOptimizer::new(false);
        let url = "https://github.com/user/repo/releases/download/v1.0/file.zip";
        let result = optimizer.optimize_url(url).await.unwrap();
        assert_eq!(result, url);
    }
}
