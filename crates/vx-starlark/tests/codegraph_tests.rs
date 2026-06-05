//! Tests for the `codegraph` provider.
//!
//! Verifies that the Starlark DSL loads correctly and that
//! `download_url` / `install_layout` produce expected results
//! for the codegraph-{os}-{arch} GitHub release format.

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

fn make_ctx() -> vx_starlark::ProviderContext {
    vx_starlark::ProviderContext::new("codegraph", std::env::temp_dir().join("vx-test"))
}

// ── provider loading ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_load_codegraph_provider() {
    let (star_path, _) = load_provider_content("codegraph");
    let provider = StarlarkProvider::load(&star_path).await.unwrap();
    assert_eq!(provider.name(), "codegraph");
}

// ── download_url checks ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_codegraph_download_url_windows_x64() {
    let (star_path, content) = load_provider_content("codegraph");
    let engine = StarlarkEngine::new();
    let mut ctx = make_ctx();
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.9.9")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/colbymchenry/codegraph/releases/download/v0.9.9/codegraph-win32-x64.zip"
    );
}

#[tokio::test]
async fn test_codegraph_download_url_linux_x64() {
    let (star_path, content) = load_provider_content("codegraph");
    let engine = StarlarkEngine::new();
    let mut ctx = make_ctx();
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.9.9")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/colbymchenry/codegraph/releases/download/v0.9.9/codegraph-linux-x64.tar.gz"
    );
}

#[tokio::test]
async fn test_codegraph_download_url_macos_arm64() {
    let (star_path, content) = load_provider_content("codegraph");
    let engine = StarlarkEngine::new();
    let mut ctx = make_ctx();
    ctx.platform.os = "macos".to_string();
    ctx.platform.arch = "arm64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.9.9")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/colbymchenry/codegraph/releases/download/v0.9.9/codegraph-darwin-arm64.tar.gz"
    );
}

#[tokio::test]
async fn test_codegraph_download_url_unsupported_platform() {
    let (star_path, content) = load_provider_content("codegraph");
    let engine = StarlarkEngine::new();
    let mut ctx = make_ctx();
    ctx.platform.os = "freebsd".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("0.9.9")],
        )
        .unwrap();

    assert!(result.is_null(), "Unsupported platform should return None");
}

// ── install_layout checks ───────────────────────────────────────────────────

#[tokio::test]
async fn test_codegraph_install_layout_windows_x64() {
    let (star_path, content) = load_provider_content("codegraph");
    let engine = StarlarkEngine::new();
    let mut ctx = make_ctx();
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("0.9.9")],
        )
        .unwrap();

    let obj = result.as_object().unwrap();
    assert_eq!(obj["type"].as_str().unwrap(), "archive");
    assert_eq!(obj["strip_prefix"].as_str().unwrap(), "codegraph-win32-x64");
    let paths: Vec<&str> = obj["executable_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    assert!(
        paths.contains(&"bin/codegraph.cmd"),
        "Windows layout should include bin/codegraph.cmd, got: {:?}",
        paths
    );
}

#[tokio::test]
async fn test_codegraph_install_layout_linux_x64() {
    let (star_path, content) = load_provider_content("codegraph");
    let engine = StarlarkEngine::new();
    let mut ctx = make_ctx();
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("0.9.9")],
        )
        .unwrap();

    let obj = result.as_object().unwrap();
    assert_eq!(obj["type"].as_str().unwrap(), "archive");
    assert_eq!(obj["strip_prefix"].as_str().unwrap(), "codegraph-linux-x64");
    let paths: Vec<&str> = obj["executable_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    assert!(
        paths.contains(&"bin/codegraph"),
        "Linux layout should include bin/codegraph, got: {:?}",
        paths
    );
}

#[tokio::test]
async fn test_codegraph_install_layout_macos_arm64() {
    let (star_path, content) = load_provider_content("codegraph");
    let engine = StarlarkEngine::new();
    let mut ctx = make_ctx();
    ctx.platform.os = "macos".to_string();
    ctx.platform.arch = "arm64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("0.9.9")],
        )
        .unwrap();

    let obj = result.as_object().unwrap();
    assert_eq!(obj["type"].as_str().unwrap(), "archive");
    assert_eq!(
        obj["strip_prefix"].as_str().unwrap(),
        "codegraph-darwin-arm64"
    );
    let paths: Vec<&str> = obj["executable_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    assert!(
        paths.contains(&"bin/codegraph"),
        "macOS layout should include bin/codegraph, got: {:?}",
        paths
    );
}
