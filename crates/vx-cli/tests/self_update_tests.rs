//! Unit tests for self-update command functionality
//!
//! Tests cover:
//! - Platform asset selection
//! - Version extraction from URLs
//! - Version comparison logic
//! - Checksum verification format parsing

use rstest::rstest;

/// Simulates the GitHubAsset structure for testing
#[derive(Debug, Clone)]
struct TestAsset {
    name: String,
}

/// Finds the appropriate platform asset using the same logic as the actual implementation.
/// This is a test-friendly version that accepts explicit os/arch parameters.
fn find_platform_asset_for_test<'a>(
    assets: &'a [TestAsset],
    target_os: &str,
    target_arch: &str,
) -> Option<&'a TestAsset> {
    // Define platform-specific patterns with REQUIRED and EXCLUDED patterns
    let (required_patterns, excluded_patterns): (Vec<&str>, Vec<&str>) =
        match (target_os, target_arch) {
            // Windows x86_64: must contain x86_64 AND windows, must NOT contain aarch64
            ("windows", "x86_64") => (vec!["x86_64", "windows"], vec!["aarch64", "arm64"]),
            // Windows x86: must contain i686 or win32, must NOT contain x86_64/aarch64
            ("windows", "x86") => (vec!["i686", "windows"], vec!["x86_64", "aarch64", "arm64"]),
            // Windows ARM64: must contain aarch64 AND windows
            ("windows", "aarch64") => (vec!["aarch64", "windows"], vec!["x86_64", "i686"]),
            // macOS x86_64: must contain x86_64 AND darwin/apple, must NOT contain aarch64
            ("macos", "x86_64") => (vec!["x86_64", "apple"], vec!["aarch64", "arm64"]),
            // macOS ARM64: must contain aarch64 AND darwin/apple
            ("macos", "aarch64") => (vec!["aarch64", "apple"], vec!["x86_64"]),
            // Linux x86_64: must contain x86_64 AND linux, must NOT contain aarch64
            ("linux", "x86_64") => (vec!["x86_64", "linux"], vec!["aarch64", "arm64"]),
            // Linux ARM64: must contain aarch64 AND linux
            ("linux", "aarch64") => (vec!["aarch64", "linux"], vec!["x86_64"]),
            _ => return None,
        };

    // Find matching asset: ALL required patterns must match, NO excluded patterns should match
    for asset in assets {
        let name_lower = asset.name.to_lowercase();

        let all_required_match = required_patterns
            .iter()
            .all(|pattern| name_lower.contains(pattern));
        let no_excluded_match = excluded_patterns
            .iter()
            .all(|pattern| !name_lower.contains(pattern));

        if all_required_match && no_excluded_match {
            return Some(asset);
        }
    }

    None
}

/// Creates a standard set of release assets for testing
fn create_test_assets() -> Vec<TestAsset> {
    vec![
        // Windows platforms
        TestAsset {
            name: "vx-x86_64-pc-windows-msvc.zip".to_string(),
        },
        TestAsset {
            name: "vx-aarch64-pc-windows-msvc.zip".to_string(),
        },
        // Linux platforms
        TestAsset {
            name: "vx-x86_64-unknown-linux-musl.tar.gz".to_string(),
        },
        TestAsset {
            name: "vx-aarch64-unknown-linux-musl.tar.gz".to_string(),
        },
        // macOS platforms
        TestAsset {
            name: "vx-x86_64-apple-darwin.tar.gz".to_string(),
        },
        TestAsset {
            name: "vx-aarch64-apple-darwin.tar.gz".to_string(),
        },
    ]
}

#[rstest]
#[case("windows", "x86_64", "vx-x86_64-pc-windows-msvc.zip")]
#[case("windows", "aarch64", "vx-aarch64-pc-windows-msvc.zip")]
#[case("linux", "x86_64", "vx-x86_64-unknown-linux-musl.tar.gz")]
#[case("linux", "aarch64", "vx-aarch64-unknown-linux-musl.tar.gz")]
#[case("macos", "x86_64", "vx-x86_64-apple-darwin.tar.gz")]
#[case("macos", "aarch64", "vx-aarch64-apple-darwin.tar.gz")]
fn test_find_platform_asset_selects_correct_binary(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] expected_asset: &str,
) {
    let assets = create_test_assets();
    let result = find_platform_asset_for_test(&assets, os, arch);

    assert!(result.is_some(), "Should find an asset for {}-{}", os, arch);
    assert_eq!(
        result.unwrap().name,
        expected_asset,
        "For {}-{}, expected {} but got {}",
        os,
        arch,
        expected_asset,
        result.unwrap().name
    );
}

#[rstest]
fn test_windows_x86_64_does_not_match_aarch64() {
    // This is the specific bug that was fixed:
    // On Windows x86_64, the old code would match aarch64-pc-windows-msvc.zip
    // because it only checked if "windows" was in the name
    let assets = vec![
        // Put aarch64 first to ensure we're not just getting the first match
        TestAsset {
            name: "vx-aarch64-pc-windows-msvc.zip".to_string(),
        },
        TestAsset {
            name: "vx-x86_64-pc-windows-msvc.zip".to_string(),
        },
    ];

    let result = find_platform_asset_for_test(&assets, "windows", "x86_64");

    assert!(result.is_some());
    assert_eq!(
        result.unwrap().name,
        "vx-x86_64-pc-windows-msvc.zip",
        "Windows x86_64 should NOT match aarch64 binary"
    );
}

#[rstest]
fn test_macos_x86_64_does_not_match_aarch64() {
    let assets = vec![
        // Put aarch64 first
        TestAsset {
            name: "vx-aarch64-apple-darwin.tar.gz".to_string(),
        },
        TestAsset {
            name: "vx-x86_64-apple-darwin.tar.gz".to_string(),
        },
    ];

    let result = find_platform_asset_for_test(&assets, "macos", "x86_64");

    assert!(result.is_some());
    assert_eq!(
        result.unwrap().name,
        "vx-x86_64-apple-darwin.tar.gz",
        "macOS x86_64 should NOT match aarch64 binary"
    );
}

#[rstest]
fn test_linux_x86_64_does_not_match_aarch64() {
    let assets = vec![
        // Put aarch64 first
        TestAsset {
            name: "vx-aarch64-unknown-linux-musl.tar.gz".to_string(),
        },
        TestAsset {
            name: "vx-x86_64-unknown-linux-musl.tar.gz".to_string(),
        },
    ];

    let result = find_platform_asset_for_test(&assets, "linux", "x86_64");

    assert!(result.is_some());
    assert_eq!(
        result.unwrap().name,
        "vx-x86_64-unknown-linux-musl.tar.gz",
        "Linux x86_64 should NOT match aarch64 binary"
    );
}

#[rstest]
fn test_unsupported_platform_returns_none() {
    let assets = create_test_assets();

    let result = find_platform_asset_for_test(&assets, "freebsd", "x86_64");
    assert!(result.is_none(), "Unsupported OS should return None");

    let result = find_platform_asset_for_test(&assets, "windows", "mips");
    assert!(result.is_none(), "Unsupported arch should return None");
}

#[rstest]
fn test_empty_assets_returns_none() {
    let assets: Vec<TestAsset> = vec![];

    let result = find_platform_asset_for_test(&assets, "windows", "x86_64");
    assert!(result.is_none(), "Empty assets should return None");
}

#[rstest]
fn test_case_insensitive_matching() {
    let assets = vec![
        TestAsset {
            name: "VX-X86_64-PC-WINDOWS-MSVC.ZIP".to_string(),
        },
        TestAsset {
            name: "VX-AARCH64-PC-WINDOWS-MSVC.ZIP".to_string(),
        },
    ];

    let result = find_platform_asset_for_test(&assets, "windows", "x86_64");

    assert!(result.is_some(), "Should match case-insensitively");
    assert!(
        result.unwrap().name.to_lowercase().contains("x86_64"),
        "Should find x86_64 asset"
    );
}

/// Tests for version extraction from URLs
fn extract_version_from_url(url: &str) -> String {
    // Extract version from GitHub release URL or CDN URL
    for part in url.split('/') {
        // Handle "vx-v1.2.3" format (release-please format)
        if part.starts_with("vx-v") && part.len() > 4 {
            let version_part = &part[4..];
            if version_part.chars().next().unwrap_or('a').is_ascii_digit() {
                return version_part.to_string();
            }
        }
        // Handle "v1.2.3" format
        if part.starts_with('v') && part.len() > 1 {
            let version_part = &part[1..];
            if version_part.chars().next().unwrap_or('a').is_ascii_digit() {
                return version_part.to_string();
            }
        }
        // Handle CDN URL format: "repo@vx-v1.2.3" or "repo@v1.2.3"
        if let Some(at_pos) = part.find('@') {
            let after_at = &part[at_pos + 1..];
            // Handle "@vx-v1.2.3" format
            if after_at.starts_with("vx-v") && after_at.len() > 4 {
                let version_part = &after_at[4..];
                if version_part.chars().next().unwrap_or('a').is_ascii_digit() {
                    return version_part.to_string();
                }
            }
            // Handle "@v1.2.3" format
            if after_at.starts_with('v') && after_at.len() > 1 {
                let version_part = &after_at[1..];
                if version_part.chars().next().unwrap_or('a').is_ascii_digit() {
                    return version_part.to_string();
                }
            }
        }
    }

    // Fallback
    "unknown".to_string()
}

#[rstest]
#[case(
    "https://github.com/loonghao/vx/releases/download/vx-v0.5.9/vx-x86_64-pc-windows-msvc.zip",
    "0.5.9"
)]
#[case(
    "https://cdn.jsdelivr.net/gh/loonghao/vx@vx-v0.5.9/vx-x86_64-pc-windows-msvc.zip",
    "0.5.9"
)]
#[case(
    "https://github.com/loonghao/vx/releases/download/v1.0.0/vx-x86_64-pc-windows-msvc.zip",
    "1.0.0"
)]
#[case(
    "https://cdn.jsdelivr.net/gh/loonghao/vx@v1.0.0/vx-x86_64-pc-windows-msvc.zip",
    "1.0.0"
)]
fn test_extract_version_from_url(#[case] url: &str, #[case] expected_version: &str) {
    let version = extract_version_from_url(url);
    assert_eq!(
        version, expected_version,
        "Failed to extract version from URL: {}",
        url
    );
}

// ============================================================================
// Version comparison tests
// ============================================================================

// Note: These tests now use vx_core::version_utils directly to ensure
// consistency with the production code in self_update.rs

#[rstest]
#[case("1.0.0", "0.9.9", true)]
#[case("0.5.29", "0.5.28", true)]
#[case("1.0.0", "0.99.99", true)]
#[case("2.0.0", "1.99.99", true)]
#[case("0.6.0", "0.5.99", true)]
#[case("0.5.28", "0.5.29", false)]
#[case("0.5.28", "0.5.28", false)]
#[case("0.4.0", "0.5.0", false)]
#[case("0.5.28-beta", "0.5.28", false)] // Pre-release handling: stable is newer
// Test prefixed version formats (vx-v, x-v, v)
#[case("vx-v0.6.27", "vx-v0.6.26", true)]
#[case("vx-v0.6.27", "v0.6.26", true)]
#[case("vx-v0.6.27", "0.6.26", true)]
#[case("v0.6.27", "vx-v0.6.26", true)]
#[case("x-v0.6.27", "vx-v0.6.26", true)]
// Test the specific bug case: 0.6.27 should NOT downgrade to 0.6.26
#[case("0.6.26", "0.6.27", false)]
#[case("vx-v0.6.26", "vx-v0.6.27", false)]
fn test_is_newer_version(#[case] version_a: &str, #[case] version_b: &str, #[case] expected: bool) {
    assert_eq!(
        vx_core::version_utils::is_newer_version(version_a, version_b),
        expected,
        "is_newer_version({}, {}) should be {}",
        version_a,
        version_b,
        expected
    );
}

// ============================================================================
// Semver extraction tests (for CDN version parsing)
// ============================================================================

#[rstest]
#[case("vx-v0.6.27", Some((0, 6, 27)))]
#[case("x-v0.6.27", Some((0, 6, 27)))]
#[case("v0.6.27", Some((0, 6, 27)))]
#[case("0.6.27", Some((0, 6, 27)))]
// Two-part versions (patch defaults to 0)
#[case("0.6", Some((0, 6, 0)))]
#[case("v0.6", Some((0, 6, 0)))]
#[case("vx-v0.6", Some((0, 6, 0)))]
// Pre-release versions
#[case("0.6.27-beta.1", Some((0, 6, 27)))]
#[case("vx-v0.6.27-beta.1", Some((0, 6, 27)))]
// Edge cases
#[case("1.0.0", Some((1, 0, 0)))]
#[case("10.20.30", Some((10, 20, 30)))]
// Invalid versions
#[case("invalid", None)]
#[case("", None)]
#[case("v", None)]
fn test_extract_semver(#[case] input: &str, #[case] expected: Option<(u64, u64, u64)>) {
    let parsed = vx_core::version_utils::parse_version(input);
    let result = parsed.map(|v| (v.major, v.minor, v.patch));
    assert_eq!(
        result, expected,
        "parse_version({}) should be {:?}",
        input, expected
    );
}

#[rstest]
fn test_extract_semver_for_cdn_versions() {
    // Simulate CDN version list parsing
    let cdn_versions = vec![
        "vx-v0.6.25",
        "vx-v0.6.26",
        "vx-v0.6.27",
        "vx-v0.5.28",
        "vx-v0.5.29",
    ];

    let latest = vx_core::version_utils::find_latest_version(&cdn_versions, false);

    assert_eq!(
        latest,
        Some("vx-v0.6.27"),
        "Should correctly identify latest version from CDN list"
    );
}

// ============================================================================
// Checksum parsing tests
// ============================================================================

/// Parse checksum from checksum file content
fn parse_checksum(content: &str) -> Option<String> {
    content.split_whitespace().next().map(|s| s.to_lowercase())
}

#[rstest]
#[case("abc123def456  vx-x86_64-pc-windows-msvc.zip", "abc123def456")]
#[case("ABC123DEF456  vx-x86_64-pc-windows-msvc.zip", "abc123def456")]
#[case("abc123def456", "abc123def456")]
#[case("  abc123def456  ", "abc123def456")]
fn test_parse_checksum(#[case] content: &str, #[case] expected: &str) {
    let result = parse_checksum(content);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), expected);
}

#[rstest]
fn test_parse_checksum_empty() {
    let result = parse_checksum("");
    assert!(result.is_none());

    let result = parse_checksum("   ");
    assert!(result.is_none());
}

// ============================================================================
// CDN asset creation tests
// ============================================================================

/// Create CDN-based assets for a given version (test version)
fn create_cdn_assets(_version: &str) -> Vec<TestAsset> {
    let asset_configs = vec![
        "vx-x86_64-pc-windows-msvc.zip",
        "vx-aarch64-pc-windows-msvc.zip",
        "vx-x86_64-unknown-linux-musl.tar.gz",
        "vx-aarch64-unknown-linux-musl.tar.gz",
        "vx-x86_64-apple-darwin.tar.gz",
        "vx-aarch64-apple-darwin.tar.gz",
    ];

    asset_configs
        .into_iter()
        .map(|name| TestAsset {
            name: name.to_string(),
        })
        .collect()
}

#[rstest]
fn test_create_cdn_assets() {
    let assets = create_cdn_assets("0.5.28");
    assert_eq!(assets.len(), 6);

    // Check Windows x64 asset exists
    let windows_asset = assets
        .iter()
        .find(|a| a.name.contains("windows") && a.name.contains("x86_64"));
    assert!(windows_asset.is_some());

    // Check Linux asset exists
    let linux_asset = assets.iter().find(|a| a.name.contains("linux"));
    assert!(linux_asset.is_some());

    // Check macOS asset exists
    let macos_asset = assets.iter().find(|a| a.name.contains("apple"));
    assert!(macos_asset.is_some());
}

// ============================================================================
// Version normalization tests
// ============================================================================

#[rstest]
#[case("vx-v0.5.28", "0.5.28")]
#[case("x-v0.5.28", "0.5.28")]
#[case("v0.5.28", "0.5.28")]
#[case("0.5.28", "0.5.28")]
#[case("vx-v1.0.0-beta.1", "1.0.0-beta.1")]
fn test_normalize_version(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(vx_core::version_utils::normalize_version(input), expected);
}

// ============================================================================
// Prerelease detection tests
// ============================================================================

#[rstest]
#[case("vx-v0.5.28", false)]
#[case("x-v0.5.28", false)]
#[case("v0.5.28", false)]
#[case("0.5.28", false)]
#[case("0.5.28-alpha.1", true)]
#[case("0.5.28-beta.1", true)]
#[case("0.5.28-rc.1", true)]
#[case("0.5.28-dev", true)]
#[case("0.5.28-pre.1", true)]
fn test_is_prerelease(#[case] version: &str, #[case] expected: bool) {
    assert_eq!(
        vx_core::version_utils::is_prerelease(version),
        expected,
        "is_prerelease({}) should be {}",
        version,
        expected
    );
}

// ============================================================================
// Download channel URL generation tests
// ============================================================================

/// Generate download URLs for different channels
fn generate_download_urls(version: &str, asset_name: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "GitHub Releases",
            format!(
                "https://github.com/loonghao/vx/releases/download/vx-v{}/{}",
                version, asset_name
            ),
        ),
        (
            "jsDelivr CDN",
            format!(
                "https://cdn.jsdelivr.net/gh/loonghao/vx@vx-v{}/{}",
                version, asset_name
            ),
        ),
        (
            "Fastly CDN",
            format!(
                "https://fastly.jsdelivr.net/gh/loonghao/vx@vx-v{}/{}",
                version, asset_name
            ),
        ),
    ]
}

#[rstest]
fn test_generate_download_urls() {
    let urls = generate_download_urls("0.5.28", "vx-x86_64-pc-windows-msvc.zip");

    assert_eq!(urls.len(), 3);

    // Check GitHub URL
    assert!(urls[0].1.contains("github.com"));
    assert!(urls[0].1.contains("vx-v0.5.28"));

    // Check jsDelivr URL
    assert!(urls[1].1.contains("cdn.jsdelivr.net"));
    assert!(urls[1].1.contains("@vx-v0.5.28"));

    // Check Fastly URL
    assert!(urls[2].1.contains("fastly.jsdelivr.net"));
    assert!(urls[2].1.contains("@vx-v0.5.28"));
}

// ============================================================================
// Checksum file URL generation tests
// ============================================================================

/// Generate checksum file URLs
fn generate_checksum_urls(version: &str, asset_name: &str) -> Vec<String> {
    let checksum_filename = format!("{}.sha256", asset_name);
    vec![
        format!(
            "https://github.com/loonghao/vx/releases/download/vx-v{}/{}",
            version, checksum_filename
        ),
        format!(
            "https://cdn.jsdelivr.net/gh/loonghao/vx@vx-v{}/{}",
            version, checksum_filename
        ),
    ]
}

#[rstest]
fn test_generate_checksum_urls() {
    let urls = generate_checksum_urls("0.5.28", "vx-x86_64-pc-windows-msvc.zip");

    assert_eq!(urls.len(), 2);

    // All URLs should end with .sha256
    for url in &urls {
        assert!(url.ends_with(".sha256"));
        assert!(url.contains("vx-x86_64-pc-windows-msvc.zip.sha256"));
    }
}

// ============================================================================
// Regression tests for fix/python-env-and-self-update branch
// ============================================================================

/// Regression test: The specific bug case where 0.6.27 was incorrectly
/// considered NOT newer than 0.6.27-beta.1
#[test]
fn test_regression_self_update_stable_vs_prerelease() {
    // Bug scenario: User has 0.6.27-beta.1 installed, release 0.6.27 is available
    // The update check should show 0.6.27 as an available update
    let current_version = "0.6.27-beta.1";
    let available_version = "0.6.27";

    assert!(
        vx_core::version_utils::is_newer_version(available_version, current_version),
        "Stable release 0.6.27 should be newer than prerelease 0.6.27-beta.1"
    );
}

/// Regression test: CDN version list parsing should correctly identify latest
#[test]
fn test_regression_cdn_version_list_latest_selection() {
    // Simulates the jsDelivr API response order (may not be semver sorted)
    let cdn_versions = vec![
        "vx-v0.6.25",
        "vx-v0.6.27", // This is the latest stable
        "vx-v0.6.26",
        "vx-v0.6.28-beta.1", // This is a newer prerelease
        "vx-v0.5.30",
    ];

    // Should find 0.6.28-beta.1 if including prereleases
    let latest_all = vx_core::version_utils::find_latest_version(&cdn_versions, false);
    assert_eq!(latest_all, Some("vx-v0.6.28-beta.1"));

    // Should find 0.6.27 if excluding prereleases (typical for stable updates)
    let latest_stable = vx_core::version_utils::find_latest_version(&cdn_versions, true);
    assert_eq!(latest_stable, Some("vx-v0.6.27"));
}

/// Regression test: Version comparison should handle mixed prefix formats
/// Bug: self_update could fail when comparing "vx-v0.6.27" with "0.6.26"
#[test]
fn test_regression_mixed_prefix_update_check() {
    // GitHub API returns "vx-v" format, local version might be stored without prefix
    assert!(vx_core::version_utils::is_newer_version(
        "vx-v0.6.27",
        "0.6.26"
    ));
    assert!(vx_core::version_utils::is_newer_version(
        "0.6.27",
        "vx-v0.6.26"
    ));

    // Same version with different formats should NOT trigger update
    assert!(!vx_core::version_utils::is_newer_version(
        "vx-v0.6.27",
        "0.6.27"
    ));
    assert!(!vx_core::version_utils::is_newer_version(
        "0.6.27",
        "vx-v0.6.27"
    ));
}

/// Regression test: Prerelease alphabetical ordering
/// Bug: rc.1 should be considered later than beta.1 for same base version
#[test]
fn test_regression_prerelease_progression() {
    // Typical prerelease progression: alpha -> beta -> rc -> stable
    assert!(vx_core::version_utils::is_newer_version(
        "0.6.27-beta.1",
        "0.6.27-alpha.1"
    ));
    assert!(vx_core::version_utils::is_newer_version(
        "0.6.27-rc.1",
        "0.6.27-beta.1"
    ));
    assert!(vx_core::version_utils::is_newer_version(
        "0.6.27",
        "0.6.27-rc.1"
    ));
}

/// Regression test: is_prerelease should correctly identify prerelease even with prefixes
#[test]
fn test_regression_vx_prefix_is_stable() {
    // Stable versions (no prerelease suffix) should NOT be considered prerelease
    assert!(!vx_core::version_utils::is_prerelease("vx-v0.6.27"));
    assert!(!vx_core::version_utils::is_prerelease("vx-v1.0.0"));
    assert!(!vx_core::version_utils::is_prerelease("x-v0.6.27"));
    assert!(!vx_core::version_utils::is_prerelease("v0.6.27"));
    assert!(!vx_core::version_utils::is_prerelease("0.6.27"));

    // Prerelease versions SHOULD be detected even with vx-v prefix
    assert!(vx_core::version_utils::is_prerelease("vx-v0.6.27-beta.1"));
    assert!(vx_core::version_utils::is_prerelease("vx-v0.6.27-alpha.1"));
    assert!(vx_core::version_utils::is_prerelease("vx-v0.6.27-rc.1"));
}

/// Regression test: Version extraction should handle edge cases
#[test]
fn test_regression_version_extraction_edge_cases() {
    // Single digit versions
    let v = vx_core::version_utils::parse_version("1.0.0").unwrap();
    assert_eq!((v.major, v.minor, v.patch), (1, 0, 0));

    // Large version numbers
    let v = vx_core::version_utils::parse_version("20.10.15").unwrap();
    assert_eq!((v.major, v.minor, v.patch), (20, 10, 15));

    // Two-part version (Node.js style "20.10")
    let v = vx_core::version_utils::parse_version("20.10").unwrap();
    assert_eq!((v.major, v.minor, v.patch), (20, 10, 0));
}
