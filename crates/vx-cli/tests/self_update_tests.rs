//! Unit tests for self-update command functionality

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
        // Handle "@vx-v1.2.3" format (CDN URL)
        if part.starts_with("@vx-v") && part.len() > 5 {
            let version_part = &part[5..];
            if version_part.chars().next().unwrap_or('a').is_ascii_digit() {
                return version_part.to_string();
            }
        }
        // Handle "@v1.2.3" format (CDN URL)
        if part.starts_with("@v") && part.len() > 2 {
            let version_part = &part[2..];
            if version_part.chars().next().unwrap_or('a').is_ascii_digit() {
                return version_part.to_string();
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
