//! Tests for RFC 0040: version_info() in vx-starlark layer.
//!
//! Covers:
//! - `VersionInfoResult::from_json()` parsing (null, valid, partial, edge cases)
//! - `StarlarkProvider::version_info()` integration (defined / not defined / None return)

use rstest::rstest;
use serde_json::json;
use vx_starlark::StarlarkProvider;
use vx_starlark::provider::VersionInfoResult;

// ============================================================
// VersionInfoResult::from_json() — parse Starlark return values
// ============================================================

#[test]
fn test_from_json_null_returns_none() {
    let result = VersionInfoResult::from_json(&json!(null));
    assert!(result.is_none());
}

#[test]
fn test_from_json_non_object_returns_none() {
    // Strings, numbers, arrays are not valid return values
    assert!(VersionInfoResult::from_json(&json!("1.0")).is_none());
    assert!(VersionInfoResult::from_json(&json!(42)).is_none());
    assert!(VersionInfoResult::from_json(&json!(true)).is_none());
    assert!(VersionInfoResult::from_json(&json!(["a", "b"])).is_none());
}

#[test]
fn test_from_json_full_rust_style() {
    let json = json!({
        "store_as": "1.93.1",
        "download_version": null,
        "install_params": {
            "toolchain": "1.93.1"
        }
    });

    let result = VersionInfoResult::from_json(&json).expect("should parse");
    assert_eq!(result.store_as, Some("1.93.1".to_string()));
    assert_eq!(result.download_version, None); // null → None
    assert_eq!(
        result.install_params.get("toolchain"),
        Some(&"1.93.1".to_string())
    );
}

#[test]
fn test_from_json_with_explicit_download_version() {
    let json = json!({
        "store_as": "3.12.0",
        "download_version": "20260101",
        "install_params": {
            "build_tag": "20260101"
        }
    });

    let result = VersionInfoResult::from_json(&json).expect("should parse");
    assert_eq!(result.store_as, Some("3.12.0".to_string()));
    assert_eq!(result.download_version, Some("20260101".to_string()));
    assert_eq!(
        result.install_params.get("build_tag"),
        Some(&"20260101".to_string())
    );
}

#[test]
fn test_from_json_empty_object_returns_some_with_defaults() {
    // An empty dict {} is a valid return — all fields default to None/empty
    let json = json!({});
    let result = VersionInfoResult::from_json(&json).expect("should parse");
    assert_eq!(result.store_as, None);
    assert_eq!(result.download_version, None);
    assert!(result.install_params.is_empty());
}

#[test]
fn test_from_json_missing_optional_fields() {
    // Only store_as present — download_version and install_params missing
    let json = json!({
        "store_as": "1.85.0"
    });
    let result = VersionInfoResult::from_json(&json).expect("should parse");
    assert_eq!(result.store_as, Some("1.85.0".to_string()));
    assert_eq!(result.download_version, None);
    assert!(result.install_params.is_empty());
}

#[test]
fn test_from_json_extra_fields_ignored() {
    // Unknown fields should be silently ignored
    let json = json!({
        "store_as": "1.93.1",
        "unknown_field": "ignored",
        "another_extra": 42
    });
    let result = VersionInfoResult::from_json(&json).expect("should parse");
    assert_eq!(result.store_as, Some("1.93.1".to_string()));
}

#[test]
fn test_from_json_multiple_install_params() {
    let json = json!({
        "store_as": "nightly",
        "install_params": {
            "toolchain": "nightly",
            "profile": "minimal",
            "components": "rust-src,rust-analyzer"
        }
    });

    let result = VersionInfoResult::from_json(&json).expect("should parse");
    assert_eq!(result.install_params.len(), 3);
    assert_eq!(
        result.install_params.get("toolchain"),
        Some(&"nightly".to_string())
    );
    assert_eq!(
        result.install_params.get("profile"),
        Some(&"minimal".to_string())
    );
    assert_eq!(
        result.install_params.get("components"),
        Some(&"rust-src,rust-analyzer".to_string())
    );
}

#[test]
fn test_from_json_install_params_non_string_values_filtered() {
    // install_params values that are not strings should be filtered out
    let json = json!({
        "store_as": "1.93.1",
        "install_params": {
            "toolchain": "stable",
            "number_val": 42,
            "bool_val": true,
            "null_val": null
        }
    });

    let result = VersionInfoResult::from_json(&json).expect("should parse");
    // Only the string value "toolchain" should survive
    assert_eq!(result.install_params.len(), 1);
    assert_eq!(
        result.install_params.get("toolchain"),
        Some(&"stable".to_string())
    );
}

#[test]
fn test_from_json_store_as_null_is_none() {
    // Explicit null for store_as should be treated as None
    let json = json!({
        "store_as": null,
        "download_version": "1.28.1"
    });

    let result = VersionInfoResult::from_json(&json).expect("should parse");
    assert_eq!(result.store_as, None);
    assert_eq!(result.download_version, Some("1.28.1".to_string()));
}

#[rstest]
#[case("stable", "stable")]
#[case("nightly", "nightly")]
#[case("beta", "beta")]
#[case("1.93.1", "1.93.1")]
#[case("1.85", "1.85")]
fn test_from_json_various_version_strings(#[case] version: &str, #[case] expected: &str) {
    let json = json!({
        "store_as": version,
        "install_params": {"toolchain": version}
    });

    let result = VersionInfoResult::from_json(&json).expect("should parse");
    assert_eq!(result.store_as.as_deref(), Some(expected));
    assert_eq!(
        result.install_params.get("toolchain"),
        Some(&expected.to_string())
    );
}

// ============================================================
// StarlarkProvider::version_info() — provider integration tests
// ============================================================

#[tokio::test]
async fn test_version_info_not_defined_returns_none() {
    // Provider without version_info() should return None (1:1 mapping)
    let content = r#"
name = "simple-tool"
description = "A tool without version indirection"

runtimes = [
    {"name": "simple-tool", "executable": "simple-tool"},
]
"#;

    let provider = StarlarkProvider::from_content("simple-tool", content)
        .await
        .unwrap();

    let result = provider.version_info("1.0.0").await.unwrap();
    assert!(
        result.is_none(),
        "tools without version_info should return None"
    );
}

#[tokio::test]
async fn test_version_info_returns_none_explicitly() {
    // Provider that defines version_info() but returns None
    let content = r#"
name = "direct-tool"
description = "A tool that returns None from version_info"

runtimes = [
    {"name": "direct-tool", "executable": "direct-tool"},
]

def version_info(ctx, user_version):
    return None
"#;

    let provider = StarlarkProvider::from_content("direct-tool", content)
        .await
        .unwrap();

    let result = provider.version_info("2.0.0").await.unwrap();
    assert!(result.is_none(), "explicit None return should be None");
}

#[tokio::test]
async fn test_version_info_returns_indirection() {
    // Provider that returns version indirection (like Rust)
    let content = r#"
name = "managed-tool"
description = "A tool with version indirection"

runtimes = [
    {"name": "managed-tool", "executable": "managed-tool"},
]

def version_info(ctx, user_version):
    return {
        "store_as": user_version,
        "download_version": None,
        "install_params": {
            "toolchain": user_version,
        },
    }
"#;

    let provider = StarlarkProvider::from_content("managed-tool", content)
        .await
        .unwrap();

    let result = provider.version_info("1.93.1").await.unwrap();
    assert!(result.is_some(), "version_info should return Some");

    let info = result.unwrap();
    assert_eq!(info.store_as, Some("1.93.1".to_string()));
    assert_eq!(info.download_version, None);
    assert_eq!(
        info.install_params.get("toolchain"),
        Some(&"1.93.1".to_string())
    );
}

#[tokio::test]
async fn test_version_info_with_download_version() {
    // Provider that specifies a specific download version
    let content = r#"
name = "pinned-tool"
description = "A tool that pins download version"

runtimes = [
    {"name": "pinned-tool", "executable": "pinned-tool"},
]

def version_info(ctx, user_version):
    return {
        "store_as": user_version,
        "download_version": "3.0.0",
        "install_params": {},
    }
"#;

    let provider = StarlarkProvider::from_content("pinned-tool", content)
        .await
        .unwrap();

    let result = provider.version_info("5.0.0").await.unwrap();
    let info = result.expect("should return Some");
    assert_eq!(info.store_as, Some("5.0.0".to_string()));
    assert_eq!(info.download_version, Some("3.0.0".to_string()));
    assert!(info.install_params.is_empty());
}

#[tokio::test]
async fn test_version_info_with_multiple_params() {
    // Provider that passes multiple install parameters
    let content = r#"
name = "complex-tool"
description = "A tool with multiple install params"

runtimes = [
    {"name": "complex-tool", "executable": "complex-tool"},
]

def version_info(ctx, user_version):
    return {
        "store_as": user_version,
        "download_version": None,
        "install_params": {
            "toolchain": user_version,
            "profile": "minimal",
            "target": "wasm32-unknown-unknown",
        },
    }
"#;

    let provider = StarlarkProvider::from_content("complex-tool", content)
        .await
        .unwrap();

    let result = provider.version_info("nightly").await.unwrap();
    let info = result.expect("should return Some");
    assert_eq!(info.store_as, Some("nightly".to_string()));
    assert_eq!(info.install_params.len(), 3);
    assert_eq!(
        info.install_params.get("profile"),
        Some(&"minimal".to_string())
    );
    assert_eq!(
        info.install_params.get("target"),
        Some(&"wasm32-unknown-unknown".to_string())
    );
}

#[rstest]
#[case("1.93.1")]
#[case("stable")]
#[case("nightly")]
#[case("beta")]
#[case("1.85")]
#[tokio::test]
async fn test_version_info_passes_user_version_through(#[case] version: &str) {
    // Verify the user_version parameter is correctly forwarded to Starlark
    let content = r#"
name = "echo-tool"
description = "Echoes user_version back"

runtimes = [
    {"name": "echo-tool", "executable": "echo-tool"},
]

def version_info(ctx, user_version):
    return {
        "store_as": user_version,
        "install_params": {"received": user_version},
    }
"#;

    let provider = StarlarkProvider::from_content("echo-tool", content)
        .await
        .unwrap();

    let result = provider.version_info(version).await.unwrap();
    let info = result.expect("should return Some");
    assert_eq!(info.store_as, Some(version.to_string()));
    assert_eq!(
        info.install_params.get("received"),
        Some(&version.to_string())
    );
}

#[tokio::test]
async fn test_version_info_minimal_return() {
    // Provider returns only store_as with no other fields
    let content = r#"
name = "minimal-info"
description = "Minimal version info"

runtimes = [
    {"name": "minimal-info", "executable": "minimal-info"},
]

def version_info(ctx, user_version):
    return {"store_as": user_version}
"#;

    let provider = StarlarkProvider::from_content("minimal-info", content)
        .await
        .unwrap();

    let result = provider.version_info("1.0.0").await.unwrap();
    let info = result.expect("should return Some");
    assert_eq!(info.store_as, Some("1.0.0".to_string()));
    assert_eq!(info.download_version, None);
    assert!(info.install_params.is_empty());
}
