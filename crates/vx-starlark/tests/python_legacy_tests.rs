//! Tests for Python provider legacy assets.
//!
//! Python 3.7.9 is available from the 20200822 release, but that release uses
//! the older .tar.zst asset naming and `python/install` archive layout.
//! Python 2.7 compatibility is provided via PyPy2.7 portable archives because
//! python-build-standalone does not publish CPython 2.7 artifacts.

use vx_starlark::StarlarkEngine;

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
async fn test_python37_download_url_windows_uses_legacy_asset() {
    let (star_path, content) = load_provider_content("python");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("python", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();
    ctx.version_date = Some("20200822".to_string());

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("3.7.9")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://github.com/astral-sh/python-build-standalone/releases/download/20200822/cpython-3.7.9-x86_64-pc-windows-msvc-shared-pgo-20200823T0118.tar.zst"
    );
}

#[tokio::test]
async fn test_python37_download_url_unsupported_arm64_returns_none() {
    let (star_path, content) = load_provider_content("python");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("python", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "arm64".to_string();
    ctx.version_date = Some("20200822".to_string());

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("3.7.9")],
        )
        .unwrap();

    assert!(result.is_null());
}

#[tokio::test]
async fn test_python37_install_layout_strips_legacy_install_directory() {
    let (star_path, content) = load_provider_content("python");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("python", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("3.7.9")],
        )
        .unwrap();
    let layout = result.as_object().unwrap();

    assert_eq!(layout["type"], "archive");
    assert_eq!(layout["strip_prefix"], "python/install");
    assert_eq!(layout["executable_paths"][0], "bin/python3");
}

#[tokio::test]
async fn test_python27_download_url_windows_uses_pypy_asset() {
    let (star_path, content) = load_provider_content("python");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("python", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();
    ctx.version_date = Some("pypy-7.3.20".to_string());

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("2.7.18")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://downloads.python.org/pypy/pypy2.7-v7.3.20-win64.zip"
    );
}

#[tokio::test]
async fn test_python27_download_url_macos_arm64_uses_pypy_asset() {
    let (star_path, content) = load_provider_content("python");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("python", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "macos".to_string();
    ctx.platform.arch = "arm64".to_string();
    ctx.version_date = Some("pypy-7.3.20".to_string());

    let result = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("2.7.18")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        "https://downloads.python.org/pypy/pypy2.7-v7.3.20-macos_arm64.tar.bz2"
    );
}

#[tokio::test]
async fn test_python27_install_layout_strips_pypy_directory() {
    let (star_path, content) = load_provider_content("python");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("python", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "linux".to_string();
    ctx.platform.arch = "x64".to_string();

    let result = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("2.7.18")],
        )
        .unwrap();
    let layout = result.as_object().unwrap();

    assert_eq!(layout["type"], "archive");
    assert_eq!(layout["strip_prefix"], "pypy2.7-v7.3.20-linux64");
    assert_eq!(layout["executable_paths"][0], "bin/pypy");
}

#[tokio::test]
async fn test_python27_execute_path_uses_pypy_binary() {
    let (star_path, content) = load_provider_content("python");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("python", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();
    ctx.paths = ctx.paths.with_version("2.7.18");

    let result = engine
        .call_function(
            &star_path,
            &content,
            "get_execute_path",
            &ctx,
            &[serde_json::json!("2.7.18")],
        )
        .unwrap();

    assert_eq!(
        result.as_str().unwrap(),
        &format!("{}/pypy.exe", ctx.paths.install_dir("2.7.18").display())
    );
}
