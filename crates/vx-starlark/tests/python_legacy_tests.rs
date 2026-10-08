//! Tests for Python provider legacy assets.
//!
//! Python 3.7.9 is available from the 20200822 release, but that release uses
//! the older .tar.zst asset naming and `python/install` archive layout.
//! Python 2.7 compatibility is provided via PyPy2.7 portable archives because
//! python-build-standalone does not publish CPython 2.7 artifacts.

use rstest::rstest;
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
async fn test_python37_download_url_unsupported_arm64_reports_error() {
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
        .unwrap_err();

    assert!(
        result
            .to_string()
            .contains("portable build unavailable for linux/arm64")
    );
}

#[rstest]
#[case(
    "windows",
    "8769a244cfe54c32c5253b78897a0fe82fe419dfde653b1afc3f5f20594cca89",
    "20200822",
    "20200823T0118"
)]
#[case(
    "linux",
    "c6d6256d13e929e77e7ee6e53470fe63ad19d173fee6d56bb1b2dbda67081543",
    "20200822",
    "20200823T0036"
)]
#[case(
    "macos",
    "53657e7712cc7b24491fb1fc66dcc8f47a577fc77df137178746987ba4c5afb8",
    "20200823",
    "20200823T2228"
)]
fn test_python37_pinned_catalog_without_release_discovery(
    #[case] os: &str,
    #[case] digest: &str,
    #[case] release: &str,
    #[case] timestamp: &str,
) {
    let (star_path, content) = load_provider_content("python");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("python", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = os.to_string();
    ctx.platform.arch = "x64".to_string();
    assert!(ctx.version_date.is_none());

    let url = engine
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("3.7.9")],
        )
        .unwrap();
    let url = url.as_str().unwrap();
    assert!(url.contains(&format!("/download/{release}/")), "{url}");
    assert!(url.contains(timestamp), "{url}");

    let layout = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("3.7.9")],
        )
        .unwrap();
    assert_eq!(layout["sha256"], digest);
    assert_eq!(layout["mirror_urls"], serde_json::json!([]));
}

#[rstest]
#[case("windows", vec!["Lib/site.py", "Lib/venv/__init__.py", "include/Python.h", "libs/python37.lib"])]
#[case("linux", vec!["lib/python3.7/site.py", "lib/python3.7/venv/__init__.py", "include/python3.7m/Python.h", "lib/libpython3.7m.a", "lib/libpython3.7m.so.1.0"])]
#[case("macos", vec!["lib/python3.7/site.py", "lib/python3.7/venv/__init__.py", "include/python3.7m/Python.h", "lib/libpython3.7m.a", "lib/libpython3.7m.dylib"])]
fn test_python37_completeness_requires_full_environment_and_build_support(
    #[case] os: &str,
    #[case] required_paths: Vec<&str>,
) {
    let (star_path, content) = load_provider_content("python");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("python", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = os.to_string();
    ctx.platform.arch = "x64".to_string();
    let layout = engine
        .call_function(
            &star_path,
            &content,
            "install_layout",
            &ctx,
            &[serde_json::json!("3.7.9")],
        )
        .unwrap();
    assert_eq!(layout["required_paths"], serde_json::json!(required_paths));
}

#[rstest]
#[case("download_url")]
#[case("install_layout")]
fn test_python37_other_patch_versions_are_not_substituted(#[case] function: &str) {
    let (star_path, content) = load_provider_content("python");
    let engine = StarlarkEngine::new();
    let mut ctx = vx_starlark::ProviderContext::new("python", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = "x64".to_string();
    let error = engine
        .call_function(
            &star_path,
            &content,
            function,
            &ctx,
            &[serde_json::json!("3.7.17")],
        )
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Python 3.7.17 has no pinned portable build")
    );
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
