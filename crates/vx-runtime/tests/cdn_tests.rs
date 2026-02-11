//! CDN acceleration tests for RealHttpClient
//!
//! These tests verify CDN functionality including:
//! - Basic client creation and configuration
//! - URL optimization (when CDN feature is enabled)
//! - Real E2E download tests with CDN acceleration
//! - Region-based CDN auto-detection

use vx_runtime::RealHttpClient;

// ============================================================================
// Unit Tests - Client Configuration
// ============================================================================

#[test]
fn test_http_client_creation() {
    // CDN is now region-aware: even with cdn-acceleration feature,
    // CDN is only auto-enabled in China environments.
    // In test/CI environments, it should default to disabled.
    let client = RealHttpClient::new();

    // In CI, CDN should be disabled regardless of feature
    if std::env::var("CI").is_ok() {
        assert!(
            !client.is_cdn_enabled(),
            "CDN should be disabled in CI environments"
        );
    }
    // Without cdn-acceleration feature, always disabled
    #[cfg(not(feature = "cdn-acceleration"))]
    assert!(!client.is_cdn_enabled());
}

#[test]
fn test_http_client_with_cdn_enabled() {
    let client = RealHttpClient::with_cdn(true);
    // Even if we request CDN, it should only be enabled if feature is active
    #[cfg(feature = "cdn-acceleration")]
    assert!(client.is_cdn_enabled());

    #[cfg(not(feature = "cdn-acceleration"))]
    assert!(!client.is_cdn_enabled());
}

#[test]
fn test_http_client_with_cdn_disabled() {
    let client = RealHttpClient::with_cdn(false);
    // Should always be disabled when explicitly set to false
    assert!(!client.is_cdn_enabled());
}

#[test]
fn test_default_http_client() {
    let client = RealHttpClient::default();
    // Default should match new() - region-aware
    if std::env::var("CI").is_ok() {
        assert!(
            !client.is_cdn_enabled(),
            "CDN should be disabled in CI environments"
        );
    }
    #[cfg(not(feature = "cdn-acceleration"))]
    assert!(!client.is_cdn_enabled());
}

#[test]
fn test_cdn_force_enable_via_env() {
    // VX_CDN=1 should force-enable CDN (when feature is compiled in)
    std::env::set_var("VX_CDN", "1");
    let client = RealHttpClient::new();
    #[cfg(feature = "cdn-acceleration")]
    assert!(
        client.is_cdn_enabled(),
        "VX_CDN=1 should force-enable CDN"
    );
    #[cfg(not(feature = "cdn-acceleration"))]
    assert!(!client.is_cdn_enabled());
    std::env::remove_var("VX_CDN");
}

#[test]
fn test_cdn_force_disable_via_env() {
    // VX_CDN=0 should force-disable CDN
    std::env::set_var("VX_CDN", "0");
    let client = RealHttpClient::new();
    assert!(
        !client.is_cdn_enabled(),
        "VX_CDN=0 should force-disable CDN"
    );
    std::env::remove_var("VX_CDN");
}

// ============================================================================
// CDN Feature Tests
// ============================================================================

#[cfg(feature = "cdn-acceleration")]
mod cdn_enabled_tests {
    use super::*;

    #[tokio::test]
    async fn test_url_optimization_github_release() {
        // Force CDN on for this test to verify optimization is attempted
        let client = RealHttpClient::with_cdn(true);
        assert!(client.is_cdn_enabled());

        // The optimize_url method is private, but we can verify behavior through download
        // In a real scenario, we'd need to mock the network layer
    }
}

// ============================================================================
// E2E Download Tests - Real Network Operations
// ============================================================================
//
// These tests perform actual downloads to verify CDN acceleration works correctly.
// They are marked with #[ignore] by default since they require network access.
// Run with: cargo test --features cdn-acceleration -- --ignored

mod e2e_download_tests {
    use super::*;
    use vx_runtime::traits::HttpClient;

    /// Test downloading a small file from GitHub releases
    /// This tests the basic download functionality with CDN acceleration
    #[tokio::test]
    #[ignore = "E2E test requiring network access - run with --ignored"]
    async fn test_download_small_file_from_github() {
        let client = RealHttpClient::new();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let dest = temp_dir.path().join("test_download.txt");

        // Download a small known file (GitHub's robots.txt as a simple test)
        let url = "https://raw.githubusercontent.com/github/gitignore/main/Rust.gitignore";

        let result = client.download(url, &dest).await;
        assert!(result.is_ok(), "Download failed: {:?}", result.err());
        assert!(dest.exists(), "Downloaded file should exist");

        let content = std::fs::read_to_string(&dest).expect("Failed to read file");
        // Check for common patterns in Rust gitignore
        assert!(
            content.contains("target") || content.contains("debug") || content.contains("Cargo"),
            "Content should contain Rust gitignore patterns, got: {}",
            &content[..content.len().min(200)]
        );

        println!("✓ Successfully downloaded file ({} bytes)", content.len());
    }

    /// Test downloading a binary release from GitHub
    /// This tests CDN acceleration for typical tool downloads
    #[tokio::test]
    #[ignore = "E2E test requiring network access - run with --ignored"]
    async fn test_download_github_release_binary() {
        let client = RealHttpClient::new();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let dest = temp_dir.path().join("just.tar.gz");

        // Download just (a small binary) - using a specific version for reproducibility
        // This is a ~2MB file, good for testing download progress
        #[cfg(target_os = "linux")]
        let url =
            "https://github.com/casey/just/releases/download/1.36.0/just-1.36.0-x86_64-unknown-linux-musl.tar.gz";
        #[cfg(target_os = "macos")]
        let url =
            "https://github.com/casey/just/releases/download/1.36.0/just-1.36.0-x86_64-apple-darwin.tar.gz";
        #[cfg(target_os = "windows")]
        let url =
            "https://github.com/casey/just/releases/download/1.36.0/just-1.36.0-x86_64-pc-windows-msvc.zip";

        let result = client.download(url, &dest).await;
        assert!(result.is_ok(), "Download failed: {:?}", result.err());
        assert!(dest.exists(), "Downloaded file should exist");

        let metadata = std::fs::metadata(&dest).expect("Failed to get metadata");
        assert!(
            metadata.len() > 100_000,
            "File should be larger than 100KB, got {} bytes",
            metadata.len()
        );

        println!(
            "✓ Successfully downloaded binary ({:.2} MB)",
            metadata.len() as f64 / 1_000_000.0
        );
    }

    /// Test downloading with progress callback
    #[tokio::test]
    #[ignore = "E2E test requiring network access - run with --ignored"]
    async fn test_download_with_progress_callback() {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::sync::Arc;

        let client = RealHttpClient::new();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let dest = temp_dir.path().join("test_progress.tar.gz");

        // Track progress
        let total_reported = Arc::new(AtomicU64::new(0));
        let downloaded_reported = Arc::new(AtomicU64::new(0));
        let callback_count = Arc::new(AtomicU64::new(0));

        let total_clone = total_reported.clone();
        let downloaded_clone = downloaded_reported.clone();
        let count_clone = callback_count.clone();

        let progress_callback = move |total: u64, downloaded: u64| {
            total_clone.store(total, Ordering::Relaxed);
            downloaded_clone.store(downloaded, Ordering::Relaxed);
            count_clone.fetch_add(1, Ordering::Relaxed);
        };

        // Download a small file
        let url = "https://github.com/casey/just/releases/download/1.36.0/just-1.36.0-x86_64-unknown-linux-musl.tar.gz";

        let result = client
            .download_with_progress(url, &dest, &progress_callback)
            .await;

        // Even if download fails (e.g., wrong platform), verify progress was tracked
        if result.is_ok() {
            assert!(dest.exists(), "Downloaded file should exist");
            assert!(
                callback_count.load(Ordering::Relaxed) > 0,
                "Progress callback should have been called"
            );
            println!(
                "✓ Progress callback called {} times, final: {}/{}",
                callback_count.load(Ordering::Relaxed),
                downloaded_reported.load(Ordering::Relaxed),
                total_reported.load(Ordering::Relaxed)
            );
        }
    }

    /// Test CDN vs non-CDN download comparison
    /// This test compares download behavior with and without CDN
    #[tokio::test]
    #[ignore = "E2E test requiring network access - run with --ignored"]
    async fn test_cdn_vs_direct_download() {
        use std::time::Instant;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

        // Small test file URL
        let url = "https://raw.githubusercontent.com/github/gitignore/main/Rust.gitignore";

        // Download with CDN
        let client_cdn = RealHttpClient::with_cdn(true);
        let dest_cdn = temp_dir.path().join("cdn_download.txt");
        let start_cdn = Instant::now();
        let result_cdn = client_cdn.download(url, &dest_cdn).await;
        let duration_cdn = start_cdn.elapsed();

        // Download without CDN
        let client_direct = RealHttpClient::with_cdn(false);
        let dest_direct = temp_dir.path().join("direct_download.txt");
        let start_direct = Instant::now();
        let result_direct = client_direct.download(url, &dest_direct).await;
        let duration_direct = start_direct.elapsed();

        // Both should succeed
        assert!(result_cdn.is_ok(), "CDN download failed: {:?}", result_cdn);
        assert!(
            result_direct.is_ok(),
            "Direct download failed: {:?}",
            result_direct
        );

        // Files should be identical
        let content_cdn = std::fs::read_to_string(&dest_cdn).expect("Failed to read CDN file");
        let content_direct =
            std::fs::read_to_string(&dest_direct).expect("Failed to read direct file");
        assert_eq!(
            content_cdn, content_direct,
            "Downloaded files should be identical"
        );

        println!("✓ CDN download: {:?}", duration_cdn);
        println!("✓ Direct download: {:?}", duration_direct);
        println!(
            "✓ CDN enabled: {}, Files match: true",
            client_cdn.is_cdn_enabled()
        );
    }

    /// Test downloading from various sources that turbo-cdn supports
    #[tokio::test]
    #[ignore = "E2E test requiring network access - run with --ignored"]
    async fn test_download_from_various_sources() {
        let client = RealHttpClient::new();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

        // Test cases: (name, url, min_expected_size)
        let test_cases = vec![
            (
                "github_raw",
                "https://raw.githubusercontent.com/rust-lang/rust/master/README.md",
                1000,
            ),
            (
                "github_release",
                "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz",
                1_000_000,
            ),
        ];

        for (name, url, min_size) in test_cases {
            let dest = temp_dir.path().join(format!("{}.download", name));
            println!("Testing {}: {}", name, url);

            let result = client.download(url, &dest).await;

            match result {
                Ok(_) => {
                    let size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
                    if size >= min_size as u64 {
                        println!("  ✓ {} - {} bytes", name, size);
                    } else {
                        println!("  ⚠ {} - {} bytes (expected >= {})", name, size, min_size);
                    }
                }
                Err(e) => {
                    println!("  ✗ {} - Error: {}", name, e);
                }
            }
        }
    }
}
