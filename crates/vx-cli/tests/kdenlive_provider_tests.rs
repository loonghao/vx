//! Contract tests for the official kdenlive Provider distribution.

use std::path::PathBuf;

use rstest::rstest;
use serde_json::{Value, json};

use vx_starlark::{ProviderContext, StarlarkEngine, StarlarkProvider};

const PROVIDER: &str = "kdenlive";

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
#[case("kdenlive")]
#[tokio::test]
async fn test_kdenlive_runtime_registration(#[case] runtime: &str) {
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
    "26.08.1",
    "https://cdn.download.kde.org/stable/kdenlive/26.08/windows/kdenlive-26.08.1_standalone.exe"
)]
#[case(
    "linux",
    "x64",
    "26.08.1",
    "https://cdn.download.kde.org/stable/kdenlive/26.08/linux/kdenlive-26.08.1-x86_64.AppImage"
)]
#[case(
    "windows",
    "x64",
    "26.08.2",
    "https://cdn.download.kde.org/stable/kdenlive/26.08/windows/kdenlive-26.08.2_standalone.exe"
)]
#[case(
    "linux",
    "x64",
    "26.08.2",
    "https://cdn.download.kde.org/stable/kdenlive/26.08/linux/kdenlive-26.08.2-x86_64.AppImage"
)]
fn test_kdenlive_official_versioned_download(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] expected: &str,
) {
    let ctx = context(os, arch, version);
    assert_eq!(call("download_url", &ctx, &[json!(version)]), expected);
}

#[rstest]
#[case("windows", "arm64", "26.08.1")]
#[case("linux", "arm64", "26.08.1")]
#[case("macos", "arm64", "26.08.1")]
#[case("windows", "x64", "26.07.80")]
#[case("windows", "x64", "26.07.90")]
#[case("windows", "x64", "26.08.80")]
#[case("linux", "x64", "26.08.90")]
fn test_kdenlive_unsupported_download_has_no_layout(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    let ctx = context(os, arch, version);
    assert!(call("download_url", &ctx, &[json!(version)]).is_null());
    assert!(call("install_layout", &ctx, &[json!(version)]).is_null());
}

#[rstest]
#[case(
    "windows",
    "x64",
    "26.08.1",
    "kdenlive-26.08.1_standalone",
    "bin/kdenlive.exe"
)]
fn test_kdenlive_verified_archive_layout_matches_execution(
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
#[case("26.08.1")]
fn test_kdenlive_linux_appimage_is_a_binary_layout(#[case] version: &str) {
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
#[case("kdenlive")]
fn test_kdenlive_system_package_reaches_the_runtime_descriptor_bridge(#[case] package: &str) {
    let path = provider_path();
    let content = std::fs::read_to_string(&path).unwrap();
    // The runtime builder reads a static variable before considering a callable.
    let strategy = StarlarkEngine::new()
        .get_variable(&path, &content, "system_install")
        .unwrap()
        .unwrap();
    assert_eq!(strategy["strategies"].as_array().unwrap().len(), 1);
    assert_eq!(strategy["strategies"][0]["manager"], "brew");
    assert_eq!(strategy["strategies"][0]["package"], package);
    assert_eq!(strategy["strategies"][0]["install_args"], "--cask");
    assert_eq!(strategy["strategies"][0]["platforms"], json!(["macos"]));
}

#[rstest]
#[case("linux", "x64", "-x86_64.AppImage")]
#[case("windows", "x64", "_standalone.exe")]
#[case("macos", "arm64", "-arm64.dmg")]
#[case("macos", "x64", "-x86_64.dmg")]
fn test_kdenlive_version_source_uses_published_platform_downloads(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] suffix: &str,
) {
    let ctx = context(os, arch, "26.08.1");
    let descriptor = call("fetch_versions", &ctx, &[]);
    assert_eq!(descriptor["__type"], "fetch_html_versions");
    assert_eq!(descriptor["url"], "https://kdenlive.org/download/");
    assert_eq!(
        descriptor["href_prefix"],
        "https://download.kde.org/stable/kdenlive/"
    );
    assert_eq!(descriptor["filename_prefix"], "kdenlive-");
    assert_eq!(descriptor["filename_suffix"], suffix);
    assert_eq!(descriptor["version_filter"], "numeric");
    let excluded = descriptor["exclude_version_suffixes"].as_array().unwrap();
    assert!(excluded.contains(&json!(".80")));
    assert!(excluded.contains(&json!(".90")));
    assert!(!excluded.contains(&json!(".1")));
}

#[rstest]
#[case("linux", "arm64")]
#[case("windows", "arm64")]
fn test_kdenlive_unsupported_platform_has_no_published_versions(
    #[case] os: &str,
    #[case] arch: &str,
) {
    let ctx = context(os, arch, "26.08.1");
    assert_eq!(call("fetch_versions", &ctx, &[]), json!([]));
}

#[rstest]
#[case(
    "windows",
    "x64",
    "26.08.1",
    "DCC_MCP_KDENLIVE_EXECUTABLE",
    "kdenlive",
    "/bin/kdenlive.exe"
)]
#[case(
    "linux",
    "x64",
    "26.08.1",
    "DCC_MCP_KDENLIVE_EXECUTABLE",
    "kdenlive",
    "/bin/kdenlive"
)]
#[case(
    "macos",
    "arm64",
    "26.08.1",
    "DCC_MCP_KDENLIVE_EXECUTABLE",
    "kdenlive",
    "/Applications/kdenlive.app/Contents/MacOS/kdenlive"
)]
fn test_kdenlive_adapter_environment_matches_resolved_executable(
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
    if os == "macos" {
        assert!(path.is_none());
        assert_eq!(operation["value"], suffix);
    } else {
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
}

#[test]
fn test_kdenlive_windows_exports_verified_bundled_render_dependencies() {
    let ctx = context("windows", "x64", "26.08.1");
    let operations = call("environment", &ctx, &[json!("26.08.1")]);
    let operations = operations.as_array().unwrap();
    let layout = call("install_layout", &ctx, &[json!("26.08.1")]);
    for (variable, binary) in [
        ("DCC_MCP_KDENLIVE_MELT", "melt"),
        ("DCC_MCP_KDENLIVE_FFPROBE", "ffprobe"),
        ("DCC_MCP_KDENLIVE_FFMPEG", "ffmpeg"),
    ] {
        let operation = operations
            .iter()
            .find(|operation| operation["key"] == variable)
            .unwrap();
        assert_eq!(operation["op"], "set");
        assert!(
            operation["value"]
                .as_str()
                .unwrap()
                .ends_with(&format!("/bin/{binary}.exe"))
        );
        assert!(
            layout["required_paths"]
                .as_array()
                .unwrap()
                .contains(&json!(format!("bin/{binary}.exe")))
        );
    }
}

#[rstest]
#[case("linux", "x64")]
#[case("macos", "arm64")]
fn test_kdenlive_does_not_invent_unix_renderer_paths(#[case] os: &str, #[case] arch: &str) {
    let ctx = context(os, arch, "26.08.1");
    let operations = call("environment", &ctx, &[json!("26.08.1")]);
    assert!(!operations.as_array().unwrap().iter().any(|operation| {
        matches!(
            operation["key"].as_str(),
            Some("DCC_MCP_KDENLIVE_MELT" | "DCC_MCP_KDENLIVE_FFPROBE" | "DCC_MCP_KDENLIVE_FFMPEG")
        )
    }));
}

#[rstest]
#[case("linux", "arm64", "26.08.1")]
fn test_kdenlive_adapter_environment_rejects_unsupported_platform(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    let ctx = context(os, arch, version);
    assert_eq!(call("environment", &ctx, &[json!(version)]), json!([]));
}
