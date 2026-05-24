//! Tests for the `mcpcall` provider.
//!
//! Verifies the component-prefixed GitHub release layout and direct-binary
//! install descriptors used by the loonghao/mcpcall v0.3.0 assets.

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
async fn test_load_mcpcall_provider() {
    let (star_path, _) = load_provider_content("mcpcall");
    let provider = StarlarkProvider::load(&star_path).await.unwrap();
    assert_eq!(provider.name(), "mcpcall");
}

#[tokio::test]
async fn test_mcpcall_download_url_windows() {
    let (star_path, content) = load_provider_content("mcpcall");
    let engine = StarlarkEngine::new();
    let mut ctx =
        vx_starlark::ProviderContext::new("mcpcall", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.3.0")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/loonghao/mcpcall/releases/download/mcpcall-v0.3.0/mcpcall-windows-x86_64.exe"
    );
}

#[tokio::test]
async fn test_mcpcall_download_url_linux_arm64() {
    let (star_path, content) = load_provider_content("mcpcall");
    let engine = StarlarkEngine::new();
    let mut ctx =
        vx_starlark::ProviderContext::new("mcpcall", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "arm64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.3.0")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/loonghao/mcpcall/releases/download/mcpcall-v0.3.0/mcpcall-linux-aarch64"
    );
}

#[tokio::test]
async fn test_mcpcall_install_layout_normalizes_binary_name() {
    let (star_path, content) = load_provider_content("mcpcall");
    let engine = StarlarkEngine::new();
    let mut ctx =
        vx_starlark::ProviderContext::new("mcpcall", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "macos".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("0.3.0")],
        )
        .unwrap();
    let layout = result.as_object().unwrap();

    assert_eq!(layout["__type"], "binary_install");
    assert_eq!(layout["source_name"], "mcpcall-macos-x86_64");
    assert_eq!(layout["target_name"], "mcpcall");
    assert_eq!(layout["target_dir"], "bin");
    assert_eq!(layout["executable_paths"][0], "bin/mcpcall");
}
