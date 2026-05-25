//! Tests for the `wasmer` provider.
//!
//! Verifies the v7.1.0 GitHub release asset layout and install descriptors.

use vx_starlark::{StarlarkEngine, StarlarkProvider};

fn load_provider_content(provider_name: &str) -> (std::path::PathBuf, String) {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let provider_dir = manifest_dir
        .parent()
        .unwrap()
        .join("vx-providers")
        .join(provider_name);
    let star_path = provider_dir.join("provider.star");
    let content = std::fs::read_to_string(&star_path).unwrap();
    (star_path, content)
}

#[tokio::test]
async fn test_load_wasmer_provider() {
    let (star_path, _) = load_provider_content("wasmer");
    let provider = StarlarkProvider::load(&star_path).await.unwrap();
    assert_eq!(provider.name(), "wasmer");
}

#[tokio::test]
async fn test_wasmer_download_url_windows_archive() {
    let (star_path, content) = load_provider_content("wasmer");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("wasmer", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("7.1.0")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/wasmerio/wasmer/releases/download/v7.1.0/wasmer-windows-amd64.tar.gz"
    );
}

#[tokio::test]
async fn test_wasmer_download_url_linux_arm64_archive() {
    let (star_path, content) = load_provider_content("wasmer");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("wasmer", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "arm64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("7.1.0")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/wasmerio/wasmer/releases/download/v7.1.0/wasmer-linux-aarch64.tar.gz"
    );
}

#[tokio::test]
async fn test_wasmer_download_url_unsupported_windows_arm64() {
    let (star_path, content) = load_provider_content("wasmer");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("wasmer", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "arm64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("7.1.0")],
        )
        .unwrap();

    assert!(result.is_null());
}

#[tokio::test]
async fn test_wasmer_install_layout_windows_archive() {
    let (star_path, content) = load_provider_content("wasmer");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("wasmer", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("7.1.0")],
        )
        .unwrap();
    let layout = result.as_object().unwrap();

    assert_eq!(layout["__type"], "archive");
    assert_eq!(layout["strip_prefix"], "");
    assert_eq!(layout["executable_paths"][0], "bin/wasmer.exe");
}

#[tokio::test]
async fn test_wasmer_install_layout_unix_archive() {
    let (star_path, content) = load_provider_content("wasmer");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("wasmer", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("7.1.0")],
        )
        .unwrap();
    let layout = result.as_object().unwrap();

    assert_eq!(layout["__type"], "archive");
    assert_eq!(layout["strip_prefix"], "");
    assert_eq!(layout["executable_paths"][0], "bin/wasmer");
}
