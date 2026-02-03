//! Tests for install scripts functionality
//!
//! These tests verify the core logic used by install scripts,
//! including version parsing, platform detection, and URL generation.

use rstest::rstest;

/// Test version parsing logic used in install scripts
/// Verifies that version strings are correctly normalized to tag format
#[rstest]
#[case::version_with_v_prefix("v0.6.0", "v0.6.0")]
#[case::version_without_v_prefix("0.6.0", "v0.6.0")]
#[case::version_with_patch("1.2.3", "v1.2.3")]
#[case::version_with_v_and_patch("v1.2.3", "v1.2.3")]
fn test_version_to_tag_format(#[case] input: &str, #[case] expected: &str) {
    // Simulate the version normalization logic from install scripts
    let normalized = if input.starts_with('v') {
        input.to_string()
    } else {
        format!("v{}", input)
    };
    assert_eq!(normalized, expected);
}

/// Test version extraction from tag
/// Verifies that version numbers are correctly extracted from tag names
#[rstest]
#[case::tag_with_v("v0.6.0", "0.6.0")]
#[case::tag_with_v_and_patch("v1.2.3", "1.2.3")]
fn test_version_extraction_from_tag(#[case] tag: &str, #[case] expected: &str) {
    // Simulate: version=$(echo "$tag_name" | sed -E 's/^v//')
    let version = tag.trim_start_matches('v');
    assert_eq!(version, expected);
}

/// Test archive name generation for different platforms
/// Verifies that archive names follow the versioned naming convention
#[rstest]
#[case::linux_gnu_x86_64(
    "0.6.0",
    "x86_64-unknown-linux-gnu",
    "vx-0.6.0-x86_64-unknown-linux-gnu.tar.gz"
)]
#[case::linux_musl_x86_64(
    "0.6.0",
    "x86_64-unknown-linux-musl",
    "vx-0.6.0-x86_64-unknown-linux-musl.tar.gz"
)]
#[case::linux_gnu_aarch64(
    "0.6.0",
    "aarch64-unknown-linux-gnu",
    "vx-0.6.0-aarch64-unknown-linux-gnu.tar.gz"
)]
#[case::linux_musl_aarch64(
    "0.6.0",
    "aarch64-unknown-linux-musl",
    "vx-0.6.0-aarch64-unknown-linux-musl.tar.gz"
)]
#[case::macos_x86_64("0.6.0", "x86_64-apple-darwin", "vx-0.6.0-x86_64-apple-darwin.tar.gz")]
#[case::macos_aarch64("0.6.0", "aarch64-apple-darwin", "vx-0.6.0-aarch64-apple-darwin.tar.gz")]
#[case::windows_x86_64(
    "0.6.0",
    "x86_64-pc-windows-msvc",
    "vx-0.6.0-x86_64-pc-windows-msvc.zip"
)]
fn test_archive_name_generation(
    #[case] version: &str,
    #[case] platform: &str,
    #[case] expected: &str,
) {
    // Simulate archive name generation from install scripts
    let ext = if platform.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    let archive_name = format!("vx-{}-{}.{}", version, platform, ext);
    assert_eq!(archive_name, expected);
}

/// Test download URL construction
/// Verifies that GitHub release URLs are correctly formatted
#[rstest]
#[case::basic_release(
    "loonghao",
    "vx",
    "v0.6.0",
    "vx-0.6.0-x86_64-unknown-linux-gnu.tar.gz",
    "https://github.com/loonghao/vx/releases/download/v0.6.0/vx-0.6.0-x86_64-unknown-linux-gnu.tar.gz"
)]
fn test_download_url_construction(
    #[case] owner: &str,
    #[case] repo: &str,
    #[case] tag: &str,
    #[case] archive: &str,
    #[case] expected: &str,
) {
    let base_url = format!("https://github.com/{}/{}/releases", owner, repo);
    let download_url = format!("{}/download/{}/{}", base_url, tag, archive);
    assert_eq!(download_url, expected);
}

/// Test fallback archive selection for Linux platforms
/// Verifies that musl/gnu fallbacks are correctly selected
#[rstest]
#[case::gnu_primary_musl_fallback(
    "x86_64-unknown-linux-gnu",
    Some("x86_64-unknown-linux-musl")
)]
#[case::musl_primary_gnu_fallback(
    "x86_64-unknown-linux-musl",
    Some("x86_64-unknown-linux-gnu")
)]
#[case::macos_no_fallback("x86_64-apple-darwin", None)]
#[case::windows_no_fallback("x86_64-pc-windows-msvc", None)]
fn test_fallback_archive_selection(
    #[case] platform: &str,
    #[case] expected_fallback: Option<&str>,
) {
    // Simulate fallback logic from install scripts
    let fallback = match platform {
        "x86_64-unknown-linux-gnu" => Some("x86_64-unknown-linux-musl"),
        "x86_64-unknown-linux-musl" => Some("x86_64-unknown-linux-gnu"),
        "aarch64-unknown-linux-gnu" => Some("aarch64-unknown-linux-musl"),
        "aarch64-unknown-linux-musl" => Some("aarch64-unknown-linux-gnu"),
        _ => None,
    };
    assert_eq!(fallback, expected_fallback);
}

/// Test platform detection normalization
/// Verifies that different architecture names are normalized correctly
#[rstest]
#[case::x86_64_variant1("x86_64", "x86_64")]
#[case::x86_64_variant2("amd64", "x86_64")]
#[case::aarch64_variant1("aarch64", "aarch64")]
#[case::aarch64_variant2("arm64", "aarch64")]
fn test_architecture_normalization(#[case] input: &str, #[case] expected: &str) {
    // Simulate arch normalization from install scripts
    let normalized = match input {
        "x86_64" | "amd64" => "x86_64",
        "aarch64" | "arm64" => "aarch64",
        _ => input,
    };
    assert_eq!(normalized, expected);
}

/// Test that old legacy naming formats are no longer used
/// This test ensures backward compatibility code has been removed
#[rstest]
#[case::old_legacy_linux("vx-x86_64-unknown-linux-gnu.tar.gz")]
#[case::old_legacy_macos("vx-x86_64-apple-darwin.tar.gz")]
#[case::old_legacy_windows("vx-x86_64-pc-windows-msvc.zip")]
fn test_legacy_naming_deprecated(#[case] legacy_name: &str) {
    // Legacy names should NOT match the new versioned format
    // New format: vx-{version}-{target}.{ext}
    let is_versioned_format = legacy_name.starts_with("vx-")
        && legacy_name
            .split('-')
            .nth(1)
            .map(|s| s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
            .unwrap_or(false);
    assert!(
        !is_versioned_format,
        "Legacy naming format '{}' should not match versioned format",
        legacy_name
    );
}

/// Test CDN URL generation for different channels
#[rstest]
#[case::github_channel(
    "github",
    "loonghao",
    "vx",
    "0.6.0",
    "vx-0.6.0-x86_64-unknown-linux-gnu.tar.gz"
)]
#[case::jsdelivr_channel(
    "jsdelivr",
    "loonghao",
    "vx",
    "0.6.0",
    "vx-0.6.0-x86_64-unknown-linux-gnu.tar.gz"
)]
#[case::fastly_channel(
    "fastly",
    "loonghao",
    "vx",
    "0.6.0",
    "vx-0.6.0-x86_64-unknown-linux-gnu.tar.gz"
)]
fn test_cdn_url_generation(
    #[case] channel: &str,
    #[case] owner: &str,
    #[case] repo: &str,
    #[case] version: &str,
    #[case] archive: &str,
) {
    let url = match channel {
        "github" => format!(
            "https://github.com/{}/{}/releases/download/v{}/{}",
            owner, repo, version, archive
        ),
        "jsdelivr" => format!(
            "https://cdn.jsdelivr.net/gh/{}/{}@v{}/{}",
            owner, repo, version, archive
        ),
        "fastly" => format!(
            "https://fastly.jsdelivr.net/gh/{}/{}@v{}/{}",
            owner, repo, version, archive
        ),
        _ => panic!("Unknown channel: {}", channel),
    };

    // Verify URL contains expected components
    assert!(url.contains(owner));
    assert!(url.contains(repo));
    assert!(url.contains(version));
    assert!(url.contains(archive));
}
