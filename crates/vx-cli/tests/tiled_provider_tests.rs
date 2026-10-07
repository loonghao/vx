//! Contract tests for the official tiled Provider distribution.

use std::path::PathBuf;

use rstest::rstest;
use serde_json::{Value, json};

use vx_starlark::{ProviderContext, StarlarkEngine, StarlarkProvider};

const PROVIDER: &str = "tiled";

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
#[case("tiled")]
#[tokio::test]
async fn test_tiled_runtime_registration(#[case] runtime: &str) {
    let provider = StarlarkProvider::load(provider_path()).await.unwrap();
    assert_eq!(provider.name(), PROVIDER);
    let runtime = provider
        .runtimes()
        .iter()
        .find(|entry| entry.name == runtime)
        .unwrap();
    assert_eq!(runtime.test_commands.len(), 1);
    let check = &runtime.test_commands[0];
    assert_eq!(
        serde_json::to_value(&check.check_type).unwrap(),
        "check_file"
    );
    assert_eq!(check.command, "{executable}");
    assert_eq!(
        check.name.as_deref(),
        Some("installed_application_executable")
    );
    assert!(check.expected_output.is_none());
}

#[rstest]
#[case(
    "windows",
    "x64",
    "1.12.2",
    "https://github.com/mapeditor/tiled/releases/download/v1.12.2/Tiled-1.12.2_Windows-10%2B_x86_64.msi"
)]
#[case(
    "linux",
    "x64",
    "1.12.2",
    "https://github.com/mapeditor/tiled/releases/download/v1.12.2/Tiled-1.12.2_Linux_x86_64.AppImage"
)]
#[case(
    "macos",
    "arm64",
    "1.12.2",
    "https://github.com/mapeditor/tiled/releases/download/v1.12.2/Tiled-1.12.2_macOS-13%2B.zip"
)]
#[case(
    "macos",
    "x64",
    "1.12.0",
    "https://github.com/mapeditor/tiled/releases/download/v1.12.0/Tiled-1.12.0_macOS-11%2B.zip"
)]
fn test_tiled_official_versioned_download(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] expected: &str,
) {
    let ctx = context(os, arch, version);
    assert_eq!(call("download_url", &ctx, &[json!(version)]), expected);
}

#[rstest]
#[case("windows", "arm64", "1.12.2")]
#[case("linux", "arm64", "1.12.2")]
#[case("windows", "x64", "1.11.2")]
fn test_tiled_unsupported_download_has_no_layout(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    let ctx = context(os, arch, version);
    assert!(call("download_url", &ctx, &[json!(version)]).is_null());
    assert!(call("install_layout", &ctx, &[json!(version)]).is_null());
}

#[rstest]
#[case("macos", "arm64", "1.12.2", "", "Tiled.app/Contents/MacOS/Tiled")]
fn test_tiled_verified_archive_layout_matches_execution(
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

#[test]
fn test_tiled_windows_uses_administrative_msi_extraction() {
    let ctx = context("windows", "x64", "1.12.2");
    let layout = call("install_layout", &ctx, &[json!("1.12.2")]);
    assert_eq!(layout["__type"], "msi_install");
    assert_eq!(layout["strip_prefix"], "PFiles/Tiled");
    assert_eq!(layout["executable_paths"], json!(["tiled.exe"]));
    assert_eq!(
        layout["url"],
        call("download_url", &ctx, &[json!("1.12.2")])
    );
}

#[rstest]
#[case("1.12.2")]
fn test_tiled_linux_appimage_is_a_binary_layout(#[case] version: &str) {
    let ctx = context("linux", "x64", version);
    let layout = call("install_layout", &ctx, &[json!(version)]);
    assert_eq!(layout["type"], "binary");
    assert_eq!(layout["target_name"], PROVIDER);
    assert_eq!(layout["target_dir"], "bin");
    assert_eq!(
        layout["executable_paths"],
        json!([format!("bin/{PROVIDER}")])
    );
}

#[rstest]
#[case(
    "windows",
    "x64",
    "1.12.2",
    "DCC_MCP_TILED_EXECUTABLE",
    "tiled",
    "/tiled.exe"
)]
#[case(
    "linux",
    "x64",
    "1.12.2",
    "DCC_MCP_TILED_EXECUTABLE",
    "tiled",
    "/bin/tiled"
)]
#[case(
    "macos",
    "arm64",
    "1.12.2",
    "DCC_MCP_TILED_EXECUTABLE",
    "tiled",
    "/Tiled.app/Contents/MacOS/Tiled"
)]
fn test_tiled_adapter_environment_matches_resolved_executable(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] variable: &str,
    #[case] runtime: &str,
    #[case] suffix: &str,
) {
    let ctx = context(os, arch, version);
    let operations = call("environment", &ctx, &[json!(version)]);
    let operations = operations.as_array().unwrap();
    let operation = operations
        .iter()
        .find(|operation| operation["key"] == variable)
        .unwrap();
    assert_eq!(operation["op"], "set");
    assert_eq!(
        operation["value"],
        call(
            "get_execute_path",
            &ctx.with_runtime_name(runtime),
            &[json!(version)]
        )
    );
    assert!(operation["value"].as_str().unwrap().ends_with(suffix));
    let path = operations
        .iter()
        .find(|operation| operation["key"] == "PATH");
    let path = path.unwrap();
    assert_eq!(path["op"], "prepend");
    assert_eq!(
        path["value"],
        operation["value"]
            .as_str()
            .unwrap()
            .rsplit_once('/')
            .unwrap()
            .0
    );
}

#[rstest]
#[case("windows", "arm64", "1.12.2")]
fn test_tiled_adapter_environment_rejects_unsupported_platform(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    let ctx = context(os, arch, version);
    assert_eq!(call("environment", &ctx, &[json!(version)]), json!([]));
}
