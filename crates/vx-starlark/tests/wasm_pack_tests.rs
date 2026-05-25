//! Tests for the `wasm-pack` provider.
//!
//! Verifies the v0.15.0 GitHub release asset layout and archive descriptors.

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
async fn test_load_wasm_pack_provider() {
    let (star_path, _) = load_provider_content("wasm-pack");
    let provider = StarlarkProvider::load(&star_path).await.unwrap();
    assert_eq!(provider.name(), "wasm-pack");
}

#[tokio::test]
async fn test_wasm_pack_download_url_windows() {
    let (star_path, content) = load_provider_content("wasm-pack");
    let engine = StarlarkEngine::new();
    let mut ctx =
        vx_starlark::ProviderContext::new("wasm-pack", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.15.0")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/wasm-bindgen/wasm-pack/releases/download/v0.15.0/wasm-pack-v0.15.0-x86_64-pc-windows-msvc.tar.gz"
    );
}

#[tokio::test]
async fn test_wasm_pack_download_url_macos_arm64() {
    let (star_path, content) = load_provider_content("wasm-pack");
    let engine = StarlarkEngine::new();
    let mut ctx =
        vx_starlark::ProviderContext::new("wasm-pack", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "macos".to_string();
    ctx.platform.arch = "arm64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.15.0")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/wasm-bindgen/wasm-pack/releases/download/v0.15.0/wasm-pack-v0.15.0-aarch64-apple-darwin.tar.gz"
    );
}

#[tokio::test]
async fn test_wasm_pack_download_url_unsupported_windows_arm64() {
    let (star_path, content) = load_provider_content("wasm-pack");
    let engine = StarlarkEngine::new();
    let mut ctx =
        vx_starlark::ProviderContext::new("wasm-pack", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "arm64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.15.0")],
        )
        .unwrap();

    assert!(result.is_null());
}

#[tokio::test]
async fn test_wasm_pack_install_layout_strips_archive_directory() {
    let (star_path, content) = load_provider_content("wasm-pack");
    let engine = StarlarkEngine::new();
    let mut ctx =
        vx_starlark::ProviderContext::new("wasm-pack", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("0.15.0")],
        )
        .unwrap();
    let layout = result.as_object().unwrap();

    assert_eq!(layout["__type"], "archive");
    assert_eq!(
        layout["strip_prefix"],
        "wasm-pack-v0.15.0-x86_64-unknown-linux-musl"
    );
    assert_eq!(layout["executable_paths"][0], "wasm-pack");
}
