//! Tests for the `hugo` provider (static site generator).
//!
//! Verifies that the Starlark DSL loads correctly and that
//! `download_url` / `install_layout` produce expected results.

use vx_starlark::StarlarkProvider;

/// Helper: load provider.star content for a given provider name.
fn load_provider_content(provider_name: &str) -> (std::path::PathBuf, String) {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let provider_dir = manifest_dir
        .parent() // crates/
        .unwrap()
        .join("vx-providers")
        .join(provider_name);
    let star_path = provider_dir.join("provider.star");
    let content = std::fs::read_to_string(&star_path).unwrap();
    (star_path, content)
}

#[tokio::test]
async fn test_load_hugo_provider() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let provider_dir = manifest_dir
        .parent() // crates/
        .unwrap()
        .join("vx-providers")
        .join("hugo");
    let star_path = provider_dir.join("provider.star");
    let provider = StarlarkProvider::load(&star_path).await.unwrap();
    assert_eq!(provider.name(), "hugo");
}

#[tokio::test]
async fn test_hugo_download_url() {
    let (star_path, content) = load_provider_content("hugo");
    let engine = vx_starlark::StarlarkEngine::new();
    let ctx = vx_starlark::ProviderContext::new("hugo", std::env::temp_dir().join("vx-test"));

    let result = engine.call_function(
        &star_path,
        &content,
        "download_url",
        &ctx,
        &[serde_json::json!("0.145.0")],
    );

    match result {
        Ok(json) => {
            if let Some(s) = json.as_str() {
                assert!(s.contains("hugo"), "URL should contain 'hugo': {}", s);
                assert!(
                    s.starts_with("https://"),
                    "URL should start with https://: {}",
                    s
                );
            } else if json.is_null() {
                // None = platform not supported
            }
        }
        Err(e) => {
            let err_str = e.to_string();
            assert!(err_str.contains("not found") || err_str.contains("FunctionNotFound"));
        }
    }
}

#[tokio::test]
async fn test_hugo_download_url_windows() {
    let (star_path, content) = load_provider_content("hugo");
    let engine = vx_starlark::StarlarkEngine::new();
    let ctx = vx_starlark::ProviderContext::new("hugo", std::env::temp_dir().join("vx-test"));
    // Note: ProviderContext doesn't allow setting platform directly.
    // This test verifies the function can be called; platform-specific
    // URL verification requires a mock context (future improvement).

    let result = engine.call_function(
        &star_path,
        &content,
        "download_url",
        &ctx,
        &[serde_json::json!("0.145.0")],
    );

    assert!(
        result.is_ok(),
        "download_url should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_hugo_install_layout() {
    let (star_path, content) = load_provider_content("hugo");
    let engine = vx_starlark::StarlarkEngine::new();
    let ctx = vx_starlark::ProviderContext::new("hugo", std::env::temp_dir().join("vx-test"));

    let result = engine.call_function(
        &star_path,
        &content,
        "install_layout",
        &ctx,
        &[serde_json::json!("0.145.0")],
    );

    match result {
        Ok(json) => {
            if let Some(obj) = json.as_object() {
                assert!(
                    obj.contains_key("type"),
                    "install_layout should return dict with 'type' key"
                );
            } else if json.is_null() {
                // None = platform not supported
            }
        }
        Err(e) => {
            let err_str = e.to_string();
            assert!(err_str.contains("not found") || err_str.contains("FunctionNotFound"));
        }
    }
}
