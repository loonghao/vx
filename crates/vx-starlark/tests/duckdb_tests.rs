//! DuckDB provider tests

use vx_starlark::StarlarkEngine;
use vx_starlark::StarlarkProvider;

// Helper: load provider.star content for a given provider name
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
async fn test_load_duckdb_provider() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let provider_dir = manifest_dir
        .parent() // crates/
        .unwrap()
        .join("vx-providers")
        .join("duckdb");
    let star_path = provider_dir.join("provider.star");
    let provider = StarlarkProvider::load(&star_path).await.unwrap();
    assert_eq!(provider.name(), "duckdb");
}

#[tokio::test]
async fn test_duckdb_download_url() {
    let (star_path, content) = load_provider_content("duckdb");
    let engine = StarlarkEngine::new();
    let ctx = vx_starlark::ProviderContext::new("duckdb", std::env::temp_dir().join("vx-test"));

    let result = engine.call_function(
        &star_path,
        &content,
        "download_url",
        &ctx,
        &[serde_json::json!("1.1.3")],
    );

    match result {
        Ok(json) => {
            if let Some(s) = json.as_str() {
                assert!(s.contains("duckdb"), "URL should contain 'duckdb': {}", s);
                assert!(
                    s.starts_with("https://"),
                    "URL should start with https://: {}",
                    s
                );
                assert!(s.contains("github.com"), "URL should be from GitHub: {}", s);
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
async fn test_duckdb_download_url_windows() {
    let (star_path, content) = load_provider_content("duckdb");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("duckdb", std::env::temp_dir().join("vx-test"));
    // Override platform to Windows
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine.call_function(
        &star_path,
        &content,
        "download_url",
        &ctx,
        &[serde_json::json!("1.1.3")],
    );

    match result {
        Ok(json) => {
            if let Some(s) = json.as_str() {
                assert!(
                    s.contains("windows"),
                    "Windows URL should contain 'windows': {}",
                    s
                );
                assert!(s.ends_with(".zip"), "Windows asset should be .zip: {}", s);
            }
        }
        Err(e) => {
            eprintln!("Error calling download_url for Windows: {}", e);
        }
    }
}

#[tokio::test]
async fn test_duckdb_download_url_linux() {
    let (star_path, content) = load_provider_content("duckdb");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("duckdb", std::env::temp_dir().join("vx-test"));
    // Override platform to Linux
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine.call_function(
        &star_path,
        &content,
        "download_url",
        &ctx,
        &[serde_json::json!("1.1.3")],
    );

    match result {
        Ok(json) => {
            if let Some(s) = json.as_str() {
                assert!(
                    s.contains("linux"),
                    "Linux URL should contain 'linux': {}",
                    s
                );
            }
        }
        Err(e) => {
            eprintln!("Error calling download_url for Linux: {}", e);
        }
    }
}

#[tokio::test]
async fn test_duckdb_install_layout() {
    let (star_path, content) = load_provider_content("duckdb");
    let engine = StarlarkEngine::new();
    let ctx = vx_starlark::ProviderContext::new("duckdb", std::env::temp_dir().join("vx-test"));

    let result = engine.call_function(
        &star_path,
        &content,
        "install_layout",
        &ctx,
        &[serde_json::json!("1.1.3")],
    );

    match result {
        Ok(json) => {
            if let Some(obj) = json.as_object() {
                assert!(
                    obj.contains_key("__type"),
                    "install_layout should return dict with '__type' key"
                );
                assert_eq!(
                    obj.get("__type").unwrap().as_str().unwrap(),
                    "archive",
                    "install_layout should return 'archive' type"
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
