//! Tests for CDN fallback mechanism

use vx_installer::{CdnOptimizer, OptimizedUrl};

#[tokio::test]
async fn test_cdn_disabled_no_fallback() {
    let optimizer = CdnOptimizer::new(false);
    let url = "https://github.com/user/repo/releases/download/v1.0/file.zip";

    let result = optimizer.optimize_url(url).await.unwrap();

    assert_eq!(result.primary, url);
    assert!(!result.has_fallback());
    assert_eq!(result.urls().len(), 1);
}

#[test]
fn test_optimized_url_single_url() {
    let optimized = OptimizedUrl {
        primary: "https://example.com/file.zip".to_string(),
        fallback: None,
    };

    let urls = optimized.urls();
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "https://example.com/file.zip");
}

#[test]
fn test_optimized_url_with_fallback() {
    let optimized = OptimizedUrl {
        primary: "https://cdn.example.com/file.zip".to_string(),
        fallback: Some("https://github.com/file.zip".to_string()),
    };

    assert!(optimized.has_fallback());

    let urls = optimized.urls();
    assert_eq!(urls.len(), 2);
    assert_eq!(urls[0], "https://cdn.example.com/file.zip");
    assert_eq!(urls[1], "https://github.com/file.zip");
}

#[test]
fn test_optimized_url_urls_order() {
    // Verify that primary is always first
    let optimized = OptimizedUrl {
        primary: "https://primary.com/file.zip".to_string(),
        fallback: Some("https://fallback.com/file.zip".to_string()),
    };

    let urls = optimized.urls();
    assert_eq!(
        urls[0], "https://primary.com/file.zip",
        "Primary should be first"
    );
    assert_eq!(
        urls[1], "https://fallback.com/file.zip",
        "Fallback should be second"
    );
}

#[cfg(feature = "cdn-acceleration")]
#[tokio::test]
async fn test_cdn_enabled_optimization() {
    use std::env;

    // Skip test in CI environments without CDN access
    if env::var("CI").is_ok() {
        return;
    }

    let optimizer = CdnOptimizer::new(true);
    let url = "https://github.com/user/repo/releases/download/v1.0/file.zip";

    let result = optimizer.optimize_url(url).await.unwrap();

    // Should have either:
    // 1. Optimized URL with fallback (if CDN optimization succeeded)
    // 2. Original URL without fallback (if CDN optimization failed)
    assert!(!result.primary.is_empty());

    if result.has_fallback() {
        // If optimized, fallback should be the original URL
        assert_eq!(result.fallback.as_ref().unwrap(), url);
    } else {
        // If not optimized, primary should be the original URL
        assert_eq!(result.primary, url);
    }
}
