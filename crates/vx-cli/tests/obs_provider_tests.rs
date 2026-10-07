//! Contract tests for the official obs Provider distribution.

use std::path::PathBuf;

use rstest::rstest;
use serde_json::{Value, json};

use vx_starlark::{ProviderContext, StarlarkEngine, StarlarkProvider};

const PROVIDER: &str = "obs";

fn provider_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("vx-providers")
        .join(PROVIDER)
        .join("provider.star")
}

fn context(os: &str, arch: &str, version: &str) -> ProviderContext {
    let mut ctx = ProviderContext::new(PROVIDER, std::env::temp_dir().join("vx-media-contracts"))
        .with_version(version);
    ctx.platform.os = os.into();
    ctx.platform.arch = arch.into();
    ctx
}

fn call(function: &str, ctx: &ProviderContext, args: &[Value]) -> Value {
    let path = provider_path();
    let content = std::fs::read_to_string(&path).unwrap();
    StarlarkEngine::new()
        .call_function(&path, &content, function, ctx, args)
        .unwrap_or_else(|error| panic!("{PROVIDER}.{function}: {error}"))
}

#[rstest]
#[case("obs")]
#[tokio::test]
async fn test_obs_runtime_registration(#[case] runtime: &str) {
    let provider = StarlarkProvider::load(provider_path()).await.unwrap();
    assert_eq!(provider.name(), PROVIDER);
    assert!(
        provider
            .runtimes()
            .iter()
            .any(|entry| entry.name == runtime)
    );
}

#[rstest]
#[case(
    "windows",
    "x64",
    "32.2.2",
    "https://github.com/obsproject/obs-studio/releases/download/32.2.2/OBS-Studio-32.2.2-Windows-x64.zip"
)]
#[case(
    "windows",
    "arm64",
    "32.2.2",
    "https://github.com/obsproject/obs-studio/releases/download/32.2.2/OBS-Studio-32.2.2-Windows-arm64.zip"
)]
fn test_obs_official_versioned_download(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] expected: &str,
) {
    let ctx = context(os, arch, version);
    assert_eq!(call("download_url", &ctx, &[json!(version)]), expected);
}

#[rstest]
#[case("windows", "x86", "32.2.2")]
#[case("windows", "arm64", "31.0.0")]
#[case("macos", "arm64", "32.2.2")]
#[case("linux", "x64", "32.2.2")]
fn test_obs_unsupported_download_has_no_layout(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    let ctx = context(os, arch, version);
    assert!(call("download_url", &ctx, &[json!(version)]).is_null());
    assert!(call("install_layout", &ctx, &[json!(version)]).is_null());
}

#[rstest]
#[case("windows", "x64", "32.2.2", "", "bin/64bit/obs64.exe")]
fn test_obs_verified_archive_layout_matches_execution(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] prefix: &str,
    #[case] executable: &str,
) {
    let ctx = context(os, arch, version);
    let layout = call("install_layout", &ctx, &[json!(version)]);
    assert_eq!(layout["type"], "archive");
    assert_eq!(layout["strip_prefix"], prefix);
    assert!(
        layout["executable_paths"]
            .as_array()
            .unwrap()
            .contains(&json!(executable))
    );
    let path = call("get_execute_path", &ctx, &[json!(version)]);
    assert!(path.as_str().unwrap().ends_with(&format!("/{executable}")));
}

#[rstest]
#[case("32.2.2", "obs")]
fn test_obs_macos_system_install_is_a_scoped_cask(#[case] version: &str, #[case] package: &str) {
    let ctx = context("macos", "arm64", version);
    let strategy = call("system_install", &ctx, &[]);
    assert_eq!(strategy["strategies"][0]["manager"], "brew");
    assert_eq!(strategy["strategies"][0]["package"], package);
    assert_eq!(strategy["strategies"][0]["install_args"], "--cask");
    assert_eq!(strategy["strategies"][0]["platforms"], json!(["macos"]));
    let unsupported = context("freebsd", "x64", version);
    assert_eq!(
        call("system_install", &unsupported, &[])["strategies"],
        json!([])
    );
}
