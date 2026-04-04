//! Tests for RFC 0040: Toolchain Version Indirection (VersionInfoResult)
//!
//! Covers:
//! - VersionInfoResult constructors and builders
//! - effective_store_version() logic
//! - Edge cases: channels, empty strings, special versions
//! - Multiple install_params scenarios
//! - Default trait implementation

use rstest::rstest;
use std::collections::HashMap;
use vx_runtime::VersionInfoResult;

// ============================================================
// Constructor & Default
// ============================================================

#[test]
fn test_version_info_result_new() {
    let info = VersionInfoResult::new("1.93.1");
    assert_eq!(info.store_as, Some("1.93.1".to_string()));
    assert_eq!(info.download_version, None);
    assert!(info.install_params.is_empty());
}

#[test]
fn test_version_info_result_default() {
    let info = VersionInfoResult::default();
    assert_eq!(info.store_as, None);
    assert_eq!(info.download_version, None);
    assert!(info.install_params.is_empty());
}

#[test]
fn test_version_info_result_new_into_string() {
    // Verify new() accepts Into<String> (not just &str)
    let owned = String::from("1.93.1");
    let info = VersionInfoResult::new(owned);
    assert_eq!(info.store_as, Some("1.93.1".to_string()));
}

// ============================================================
// Builder pattern
// ============================================================

#[test]
fn test_version_info_result_builder() {
    let info = VersionInfoResult::new("1.93.1")
        .with_download_version("1.28.1")
        .with_install_param("toolchain", "1.93.1");

    assert_eq!(info.store_as, Some("1.93.1".to_string()));
    assert_eq!(info.download_version, Some("1.28.1".to_string()));
    assert_eq!(
        info.install_params.get("toolchain"),
        Some(&"1.93.1".to_string())
    );
}

#[test]
fn test_builder_chaining_multiple_params() {
    let info = VersionInfoResult::new("nightly")
        .with_install_param("toolchain", "nightly")
        .with_install_param("profile", "minimal")
        .with_install_param("components", "rust-src,rust-analyzer")
        .with_install_param("target", "wasm32-unknown-unknown");

    assert_eq!(info.install_params.len(), 4);
    assert_eq!(
        info.install_params.get("toolchain"),
        Some(&"nightly".to_string())
    );
    assert_eq!(
        info.install_params.get("profile"),
        Some(&"minimal".to_string())
    );
    assert_eq!(
        info.install_params.get("components"),
        Some(&"rust-src,rust-analyzer".to_string())
    );
    assert_eq!(
        info.install_params.get("target"),
        Some(&"wasm32-unknown-unknown".to_string())
    );
}

#[test]
fn test_builder_overwrite_install_param() {
    // Later with_install_param should overwrite earlier one with same key
    let info = VersionInfoResult::new("1.93.1")
        .with_install_param("toolchain", "stable")
        .with_install_param("toolchain", "1.93.1");

    assert_eq!(
        info.install_params.get("toolchain"),
        Some(&"1.93.1".to_string())
    );
    assert_eq!(info.install_params.len(), 1);
}

#[test]
fn test_builder_download_version_overwrite() {
    // Later with_download_version should replace earlier one
    let info = VersionInfoResult::new("1.93.1")
        .with_download_version("1.27.0")
        .with_download_version("1.28.1");

    assert_eq!(info.download_version, Some("1.28.1".to_string()));
}

// ============================================================
// effective_store_version()
// ============================================================

#[test]
fn test_effective_store_version_with_store_as() {
    let info = VersionInfoResult::new("1.93.1");
    assert_eq!(info.effective_store_version("fallback"), "1.93.1");
}

#[test]
fn test_effective_store_version_fallback() {
    let info = VersionInfoResult::default(); // store_as = None
    assert_eq!(info.effective_store_version("1.93.1"), "1.93.1");
}

#[test]
fn test_effective_store_version_empty_fallback() {
    let info = VersionInfoResult::default();
    assert_eq!(info.effective_store_version(""), "");
}

#[rstest]
#[case("stable", "stable")]
#[case("nightly", "nightly")]
#[case("beta", "beta")]
#[case("1.93.1", "1.93.1")]
#[case("1.85", "1.85")]
fn test_effective_store_version_with_store_as_channels(
    #[case] store_as: &str,
    #[case] expected: &str,
) {
    let info = VersionInfoResult::new(store_as);
    assert_eq!(info.effective_store_version("unused-fallback"), expected);
}

// ============================================================
// Rust-specific scenarios
// ============================================================

#[test]
fn test_rust_toolchain_pattern() {
    // Simulates the exact Rust provider pattern
    let user_version = "1.93.1";
    let info = VersionInfoResult::new(user_version).with_install_param("toolchain", user_version);

    // download_version is None → executor should use latest rustup
    assert_eq!(info.download_version, None);
    // store_as = user_version → ~/.vx/store/rust/1.93.1/
    assert_eq!(info.effective_store_version("unused"), "1.93.1");
    // install_params.toolchain → rustup-init --default-toolchain 1.93.1
    assert_eq!(
        info.install_params.get("toolchain"),
        Some(&"1.93.1".to_string())
    );
}

#[test]
fn test_version_info_result_stable_channel() {
    // "stable" is a valid version for Rust passthrough
    let info = VersionInfoResult::new("stable").with_install_param("toolchain", "stable");
    assert_eq!(info.effective_store_version("fallback"), "stable");
    assert_eq!(
        info.install_params.get("toolchain"),
        Some(&"stable".to_string())
    );
}

#[test]
fn test_nightly_with_date() {
    // nightly-2026-04-01 as a valid version string
    let info = VersionInfoResult::new("nightly-2026-04-01")
        .with_install_param("toolchain", "nightly-2026-04-01");

    assert_eq!(info.effective_store_version("unused"), "nightly-2026-04-01");
    assert_eq!(
        info.install_params.get("toolchain"),
        Some(&"nightly-2026-04-01".to_string())
    );
}

// ============================================================
// Hypothetical future use cases
// ============================================================

#[test]
fn test_python_build_standalone_pattern() {
    // Hypothetical: Python build-standalone uses build tags as download_version
    let info = VersionInfoResult::new("3.12.0")
        .with_download_version("20260101")
        .with_install_param("build_tag", "20260101")
        .with_install_param("flavor", "shared-pgo+lto");

    assert_eq!(info.store_as, Some("3.12.0".to_string()));
    assert_eq!(info.download_version, Some("20260101".to_string()));
    assert_eq!(
        info.install_params.get("build_tag"),
        Some(&"20260101".to_string())
    );
    assert_eq!(
        info.install_params.get("flavor"),
        Some(&"shared-pgo+lto".to_string())
    );
}

#[test]
fn test_no_store_as_uses_fallback() {
    // When only download_version is set but store_as is None
    let info = VersionInfoResult {
        download_version: Some("1.28.1".to_string()),
        ..Default::default()
    };

    assert_eq!(info.store_as, None);
    assert_eq!(
        info.effective_store_version("user-specified"),
        "user-specified"
    );
}

// ============================================================
// Clone and Debug traits
// ============================================================

#[test]
fn test_version_info_result_clone() {
    let original = VersionInfoResult::new("1.93.1")
        .with_download_version("1.28.1")
        .with_install_param("toolchain", "1.93.1");

    let cloned = original.clone();
    assert_eq!(cloned.store_as, original.store_as);
    assert_eq!(cloned.download_version, original.download_version);
    assert_eq!(cloned.install_params, original.install_params);
}

#[test]
fn test_version_info_result_debug() {
    let info = VersionInfoResult::new("1.93.1");
    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("1.93.1"));
    assert!(debug_str.contains("VersionInfoResult"));
}

// ============================================================
// Direct struct construction (non-builder)
// ============================================================

#[test]
fn test_direct_struct_construction() {
    let mut params = HashMap::new();
    params.insert("key1".to_string(), "val1".to_string());
    params.insert("key2".to_string(), "val2".to_string());

    let info = VersionInfoResult {
        store_as: Some("custom".to_string()),
        download_version: Some("2.0.0".to_string()),
        install_params: params,
    };

    assert_eq!(info.store_as, Some("custom".to_string()));
    assert_eq!(info.download_version, Some("2.0.0".to_string()));
    assert_eq!(info.install_params.len(), 2);
}
