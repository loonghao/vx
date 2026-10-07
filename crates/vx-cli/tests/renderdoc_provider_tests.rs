//! Contract tests for the official renderdoc Provider distribution.

use std::path::PathBuf;

use rstest::rstest;
use serde_json::{Value, json};

use vx_starlark::{ProviderContext, StarlarkEngine, StarlarkProvider};

const PROVIDER: &str = "renderdoc";

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
    "renderdoc",
    "check_file",
    "{executable}",
    "installed_application_executable",
    None
)]
#[case(
    "renderdoccmd",
    "command",
    "{executable} version",
    "version_check",
    Some("renderdoccmd")
)]
#[tokio::test]
async fn test_renderdoc_runtime_registration(
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
    "1.46",
    "https://renderdoc.org/stable/1.46/RenderDoc_1.46_64.zip"
)]
#[case(
    "windows",
    "x86",
    "1.46",
    "https://renderdoc.org/stable/1.46/RenderDoc_1.46_32.zip"
)]
#[case(
    "linux",
    "x64",
    "1.46",
    "https://renderdoc.org/stable/1.46/renderdoc_1.46.tar.gz"
)]
fn test_renderdoc_official_versioned_download(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] expected: &str,
) {
    let ctx = context(os, arch, version);
    assert_eq!(call("download_url", &ctx, &[json!(version)]), expected);
}

#[rstest]
#[case("windows", "arm64", "1.46")]
#[case("macos", "arm64", "1.46")]
#[case("linux", "arm64", "1.46")]
fn test_renderdoc_unsupported_download_has_no_layout(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    let ctx = context(os, arch, version);
    assert!(call("download_url", &ctx, &[json!(version)]).is_null());
    assert!(call("install_layout", &ctx, &[json!(version)]).is_null());
}

#[rstest]
#[case("windows", "x64", "1.46", "RenderDoc_1.46_64", "qrenderdoc.exe")]
#[case("linux", "x64", "1.46", "renderdoc_1.46", "bin/qrenderdoc")]
fn test_renderdoc_verified_archive_layout_matches_execution(
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
#[case("renderdoccmd", "linux", "x64", "1.46", "/bin/renderdoccmd")]
#[case("renderdoccmd", "windows", "x64", "1.46", "/renderdoccmd.exe")]
fn test_renderdoc_bundled_runtime_executes_its_own_binary(
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

#[rstest]
#[case(
    "windows",
    "x64",
    "1.46",
    "DCC_MCP_RENDERDOC_CMD",
    "renderdoccmd",
    "/renderdoccmd.exe"
)]
#[case(
    "linux",
    "x64",
    "1.46",
    "DCC_MCP_RENDERDOC_CMD",
    "renderdoccmd",
    "/bin/renderdoccmd"
)]
fn test_renderdoc_adapter_environment_matches_resolved_executable(
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
#[case("macos", "arm64", "1.46")]
fn test_renderdoc_adapter_environment_rejects_unsupported_platform(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    let ctx = context(os, arch, version);
    assert_eq!(call("environment", &ctx, &[json!(version)]), json!([]));
}
