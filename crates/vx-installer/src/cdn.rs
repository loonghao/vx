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
    ///
    /// Returns a tuple of (optimized_url, original_url) to enable fallback on download failure.
    pub async fn optimize_url(&self, url: &str) -> Result<OptimizedUrl> {
        if !self.enabled {
            return Ok(OptimizedUrl {
                primary: url.to_string(),
                fallback: None,
            });
        }

        #[cfg(feature = "cdn-acceleration")]
        {
            match turbo_cdn::async_api::quick::optimize_url(url).await {
                Ok(optimized) => {
                    if optimized == url {
                        // No optimization needed
                        tracing::debug!(url = url, "URL not optimized by CDN");
                        Ok(OptimizedUrl {
                            primary: url.to_string(),
                            fallback: None,
                        })
                    } else {
                        // CDN URL available, keep original as fallback
                        tracing::debug!(
                            original = url,
                            optimized = %optimized,
                            "CDN URL optimized, original kept as fallback"
                        );
                        Ok(OptimizedUrl {
                            primary: optimized,
                            fallback: Some(url.to_string()),
                        })
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        url = url,
                        error = %e,
                        "CDN optimization failed, using original URL"
                    );
                    Ok(OptimizedUrl {
                        primary: url.to_string(),
                        fallback: None,
                    })
                }
            }
        }

        #[cfg(not(feature = "cdn-acceleration"))]
        {
            Ok(OptimizedUrl {
                primary: url.to_string(),
                fallback: None,
            })
        }
    }
}

/// Result of URL optimization
#[derive(Debug, Clone)]
pub struct OptimizedUrl {
    /// Primary URL to try first (CDN-optimized if available)
    pub primary: String,
    /// Fallback URL to try if primary fails (original URL)
    pub fallback: Option<String>,
}

impl OptimizedUrl {
    /// Get all URLs to try in order (primary first, then fallback if available)
    pub fn urls(&self) -> Vec<&str> {
        let mut urls = vec![self.primary.as_str()];
        if let Some(fallback) = &self.fallback {
            urls.push(fallback.as_str());
        }
        urls
    }

    /// Check if there's a fallback URL available
    pub fn has_fallback(&self) -> bool {
        self.fallback.is_some()
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
        assert_eq!(result.primary, url);
        assert!(!result.has_fallback());
    }

    #[test]
    fn test_optimized_url_urls() {
        let optimized = OptimizedUrl {
            primary: "https://cdn.example.com/file.zip".to_string(),
            fallback: Some("https://github.com/file.zip".to_string()),
        };
        let urls = optimized.urls();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://cdn.example.com/file.zip");
        assert_eq!(urls[1], "https://github.com/file.zip");
    }

    #[test]
    fn test_optimized_url_no_fallback() {
        let optimized = OptimizedUrl {
            primary: "https://example.com/file.zip".to_string(),
            fallback: None,
        };
        let urls = optimized.urls();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/file.zip");
        assert!(!optimized.has_fallback());
    }
}
