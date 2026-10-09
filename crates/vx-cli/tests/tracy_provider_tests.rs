//! Contract tests for the official tracy Provider distribution.

use std::path::PathBuf;

use rstest::rstest;
use serde_json::{Value, json};

use vx_starlark::{ProviderContext, StarlarkEngine, StarlarkProvider};

const PROVIDER: &str = "tracy";

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
#[case(
    "tracy",
    "check_file",
    "{executable}",
    "installed_profiler_executable",
    None
)]
#[case(
    "tracy-capture",
    "check_file",
    "{executable}",
    "installed_helper_executable",
    None
)]
#[case(
    "tracy-csvexport",
    "command",
    "{executable} --version",
    "cli_version_check",
    Some(r"tracy-csvexport \d+\.\d+\.\d+")
)]
#[case(
    "tracy-update",
    "check_file",
    "{executable}",
    "installed_helper_executable",
    None
)]
#[tokio::test]
async fn test_tracy_runtime_registration(
    #[case] runtime: &str,
    #[case] check_type: &str,
    #[case] command: &str,
    #[case] check_name: &str,
    #[case] expected_output: Option<&str>,
) {
    let provider = StarlarkProvider::load(provider_path()).await.unwrap();
    assert_eq!(provider.name(), PROVIDER);
    let runtime = provider
        .runtimes()
        .iter()
        .find(|entry| entry.name == runtime)
        .unwrap();
    assert_eq!(runtime.test_commands.len(), 1);
    let check = &runtime.test_commands[0];
    assert_eq!(serde_json::to_value(&check.check_type).unwrap(), check_type);
    assert_eq!(check.command, command);
    assert_eq!(check.name.as_deref(), Some(check_name));
    assert_eq!(check.expected_output.as_deref(), expected_output);
    assert!(check.expect_success);
}

#[rstest]
#[case(
    "windows",
    "x64",
    "0.14.1",
    "https://github.com/wolfpld/tracy/releases/download/v0.14.1/windows-0.14.1.zip"
)]
#[case(
    "linux",
    "x64",
    "0.14.1",
    "https://github.com/wolfpld/tracy/releases/download/v0.14.1/linux-0.14.1.zip"
)]
#[case(
    "macos",
    "arm64",
    "0.14.1",
    "https://github.com/wolfpld/tracy/releases/download/v0.14.1/macos-0.14.1.zip"
)]
fn test_tracy_official_versioned_download(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] expected: &str,
) {
    let ctx = context(os, arch, version);
    assert_eq!(call("download_url", &ctx, &[json!(version)]), expected);
}

#[rstest]
#[case("windows", "arm64", "0.14.1")]
#[case("linux", "arm64", "0.14.1")]
#[case("macos", "x64", "0.14.1")]
#[case("windows", "x64", "0.13.0")]
fn test_tracy_unsupported_download_has_no_layout(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    let ctx = context(os, arch, version);
    assert!(call("download_url", &ctx, &[json!(version)]).is_null());
    assert!(call("install_layout", &ctx, &[json!(version)]).is_null());
}

#[rstest]
#[case("windows", "x64", "0.14.1", "", "tracy-profiler.exe")]
#[case("linux", "x64", "0.14.1", "", "tracy-profiler-x86_64.AppImage")]
#[case(
    "macos",
    "arm64",
    "0.14.1",
    "",
    "tracy-profiler.app/Contents/MacOS/tracy-profiler"
)]
fn test_tracy_verified_archive_layout_matches_execution(
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
    assert_eq!(layout["required_paths"], layout["executable_paths"]);
    let path = call("get_execute_path", &ctx, &[json!(version)]);
    assert!(path.as_str().unwrap().ends_with(&format!("/{executable}")));
}

#[rstest]
#[case("tracy-capture", "windows", "x64", "0.14.1", "/tracy-capture.exe")]
#[case("tracy-csvexport", "linux", "x64", "0.14.1", "/tracy-csvexport")]
#[case("tracy-update", "macos", "arm64", "0.14.1", "/tracy-update")]
fn test_tracy_bundled_runtime_executes_its_own_binary(
    #[case] runtime: &str,
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] suffix: &str,
) {
    let ctx = context(os, arch, version).with_runtime_name(runtime);
    let executable = call("get_execute_path", &ctx, &[json!(version)]);
    assert!(executable.as_str().unwrap().ends_with(suffix));
}
