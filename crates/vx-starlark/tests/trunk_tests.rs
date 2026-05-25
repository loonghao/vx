//! Tests for the `trunk` provider.
//!
//! Verifies the v0.21.14 GitHub release asset layout and archive descriptors.

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
async fn test_load_trunk_provider() {
    let (star_path, _) = load_provider_content("trunk");
    let provider = StarlarkProvider::load(&star_path).await.unwrap();
    assert_eq!(provider.name(), "trunk");
}

#[tokio::test]
async fn test_trunk_download_url_windows() {
    let (star_path, content) = load_provider_content("trunk");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("trunk", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.21.14")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-pc-windows-msvc.zip"
    );
}

#[tokio::test]
async fn test_trunk_download_url_linux_arm64() {
    let (star_path, content) = load_provider_content("trunk");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("trunk", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "arm64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.21.14")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-aarch64-unknown-linux-gnu.tar.gz"
    );
}

#[tokio::test]
async fn test_trunk_download_url_unsupported_windows_arm64() {
    let (star_path, content) = load_provider_content("trunk");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("trunk", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "arm64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.21.14")],
        )
        .unwrap();

    assert!(result.is_null());
}

#[tokio::test]
async fn test_trunk_install_layout_archive_root_binary() {
    let (star_path, content) = load_provider_content("trunk");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("trunk", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "macos".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("0.21.14")],
        )
        .unwrap();
    let layout = result.as_object().unwrap();

    assert_eq!(layout["__type"], "archive");
    assert_eq!(layout["strip_prefix"], "");
    assert_eq!(layout["executable_paths"][0], "trunk");
}
