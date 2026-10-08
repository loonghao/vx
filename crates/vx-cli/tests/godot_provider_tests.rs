//! godot Provider contracts, independent of other optional DCC Providers.

use rstest::rstest;
use serde_json::json;

use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime};
use vx_starlark::StarlarkProvider;

#[path = "support/provider_contract.rs"]
mod provider_contract;

#[path = "support/managed_cache.rs"]
mod managed_cache;

use managed_cache::managed_cache_context;
use provider_contract::{call, source};

#[rstest]
#[case("x64")]
#[case("arm64")]
fn test_godot_macos_preserves_the_app_bundle_root(#[case] arch: &str) {
    let layout = call("godot", "install_layout", "macos", arch, "4.7.2-stable");
    assert!(layout.get("strip_prefix").is_none());
    assert_eq!(
        layout["required_paths"],
        json!(["Godot.app/Contents/MacOS/Godot"])
    );
}

#[rstest]
#[case("godot")]
#[tokio::test]
async fn test_load_dcc_provider_with_system_discovery(#[case] name: &str) {
    let (path, _) = source(name);
    let provider = StarlarkProvider::load(&path).await.unwrap();
    assert_eq!(provider.name(), name);
    assert_eq!(provider.runtimes().len(), 1);
    assert!(
        !provider.runtimes()[0].system_paths.is_empty(),
        "{name} must discover existing DCC-MCP application installations"
    );
}

#[tokio::test]
async fn test_godot_version_validation_is_explicitly_headless() {
    let (path, _) = source("godot");
    let provider = StarlarkProvider::load(path).await.unwrap();
    let checks = &provider.runtimes()[0].test_commands;
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].command, "{executable} --headless --version");
    assert_eq!(
        checks[0].check_type,
        vx_starlark::provider::types::TestCheckType::Command
    );
    assert!(checks[0].expect_success);
    assert_eq!(checks[0].expected_output.as_deref(), Some(r"\d+\.\d+"));
}

#[rstest]
#[case("x64", "win64.exe", true)]
#[case("x64", "win64.exe", false)]
#[case("x86", "win32.exe", true)]
#[case("x86", "win32.exe", false)]
#[case("arm64", "windows_arm64.exe", true)]
#[case("arm64", "windows_arm64.exe", false)]
#[tokio::test]
async fn test_godot_managed_cache_requires_the_complete_pair_and_executes_the_editor(
    #[case] arch: &str,
    #[case] platform: &str,
    #[case] editor_first: bool,
) {
    let (_directory, ctx) = managed_cache_context();
    let version = "4.7.2-stable";
    let layout = call("godot", "install_layout", "windows", arch, version);
    let editor = format!("Godot_v{version}_{platform}");
    let console = format!("{}_console.exe", editor.strip_suffix(".exe").unwrap());
    assert_eq!(layout["executable_paths"], json!([editor]));
    assert_eq!(layout["required_paths"], json!([editor, console]));
    let runtime = ManifestDrivenRuntime::new("godot", "godot", ProviderSource::BuiltIn)
        .with_install_layout(move |_| {
            let layout = layout.clone();
            Box::pin(async move { Ok(Some(layout)) })
        });
    let install_dir = ctx.paths.version_store_dir("godot", version);
    std::fs::create_dir_all(&install_dir).unwrap();
    assert!(!runtime.is_installed(version, &ctx).await.unwrap());
    let (first, second) = if editor_first {
        (&editor, &console)
    } else {
        (&console, &editor)
    };
    std::fs::write(install_dir.join(first), b"").unwrap();
    assert!(!runtime.is_installed(version, &ctx).await.unwrap());
    std::fs::write(install_dir.join(second), b"").unwrap();
    assert!(runtime.is_installed(version, &ctx).await.unwrap());
    let warm_entry = runtime
        .get_executable_path_for_version(version, &ctx)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(warm_entry, install_dir.join(editor));
    assert_ne!(warm_entry, install_dir.join(console));
}

#[rstest]
#[case(
    "godot",
    "windows",
    "x86",
    "4.7.2",
    "https://github.com/godotengine/godot-builds/releases/download/4.7.2-stable/Godot_v4.7.2-stable_win32.exe.zip"
)]
#[case(
    "godot",
    "windows",
    "x64",
    "4.7.2",
    "https://github.com/godotengine/godot-builds/releases/download/4.7.2-stable/Godot_v4.7.2-stable_win64.exe.zip"
)]
#[case(
    "godot",
    "windows",
    "arm64",
    "4.7.2-stable",
    "https://github.com/godotengine/godot-builds/releases/download/4.7.2-stable/Godot_v4.7.2-stable_windows_arm64.exe.zip"
)]
#[case(
    "godot",
    "macos",
    "arm64",
    "4.7.2-stable",
    "https://github.com/godotengine/godot-builds/releases/download/4.7.2-stable/Godot_v4.7.2-stable_macos.universal.zip"
)]
#[case(
    "godot",
    "linux",
    "armv7",
    "4.7.2",
    "https://github.com/godotengine/godot-builds/releases/download/4.7.2-stable/Godot_v4.7.2-stable_linux.arm32.zip"
)]
fn test_dcc_download_url_matches_official_artifact(
    #[case] provider: &str,
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] expected: &str,
) {
    assert_eq!(call(provider, "download_url", os, arch, version), expected);
}

#[rstest]
#[case(
    "godot",
    "windows",
    "x64",
    "4.7.2-stable",
    Some(""),
    "Godot_v4.7.2-stable_win64.exe"
)]
#[case(
    "godot",
    "windows",
    "x86",
    "4.7.2",
    Some(""),
    "Godot_v4.7.2-stable_win32.exe"
)]
#[case(
    "godot",
    "windows",
    "arm64",
    "4.7.2-stable",
    Some(""),
    "Godot_v4.7.2-stable_windows_arm64.exe"
)]
#[case(
    "godot",
    "macos",
    "arm64",
    "4.7.2",
    None,
    "Godot.app/Contents/MacOS/Godot"
)]
#[case(
    "godot",
    "linux",
    "arm64",
    "4.7.2",
    Some(""),
    "Godot_v4.7.2-stable_linux.arm64"
)]
fn test_dcc_archive_layout_and_execution_path_agree(
    #[case] provider: &str,
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] root: Option<&str>,
    #[case] executable: &str,
) {
    let layout = call(provider, "install_layout", os, arch, version);
    assert_eq!(layout["type"], "archive");
    assert_eq!(
        layout
            .get("strip_prefix")
            .and_then(serde_json::Value::as_str),
        root
    );
    assert!(
        layout["executable_paths"]
            .as_array()
            .unwrap()
            .contains(&json!(executable))
    );
    let path = call(provider, "get_execute_path", os, arch, version);
    assert!(path.as_str().unwrap().ends_with(&format!("/{executable}")));
}

#[rstest]
#[case("godot", "linux", "s390x", "4.7.2")]
fn test_dcc_unsupported_archives_do_not_fabricate_urls(
    #[case] provider: &str,
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    assert!(call(provider, "download_url", os, arch, version).is_null());
    assert!(call(provider, "install_layout", os, arch, version).is_null());
}
