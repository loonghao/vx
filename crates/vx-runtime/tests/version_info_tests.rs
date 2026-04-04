//! Tests for RFC 0040: Toolchain Version Indirection (VersionInfoResult)

use vx_runtime::VersionInfoResult;

#[test]
fn test_version_info_result_new() {
    let info = VersionInfoResult::new("1.93.1");
    assert_eq!(info.store_as, Some("1.93.1".to_string()));
    assert_eq!(info.download_version, None);
    assert!(info.install_params.is_empty());
}

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
fn test_version_info_result_effective_store_version_with_store_as() {
    let info = VersionInfoResult::new("1.93.1");
    assert_eq!(info.effective_store_version("fallback"), "1.93.1");
}

#[test]
fn test_version_info_result_effective_store_version_fallback() {
    let info = VersionInfoResult::default(); // store_as = None
    assert_eq!(info.effective_store_version("1.93.1"), "1.93.1");
}

#[test]
fn test_version_info_result_default() {
    let info = VersionInfoResult::default();
    assert_eq!(info.store_as, None);
    assert_eq!(info.download_version, None);
    assert!(info.install_params.is_empty());
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
