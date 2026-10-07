//! openscad Provider contracts, independent of other optional DCC Providers.

use rstest::rstest;
use serde_json::json;

use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime};
use vx_starlark::{ProviderContext, StarlarkEngine, StarlarkProvider};

#[path = "support/provider_contract.rs"]
mod provider_contract;

#[path = "support/managed_cache.rs"]
mod managed_cache;

use managed_cache::managed_cache_context;
use provider_contract::{call, source};

#[rstest]
#[case("openscad")]
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

#[rstest]
#[case(
    "openscad",
    "windows",
    "x64",
    "2021.01",
    "https://github.com/openscad/openscad/releases/download/openscad-2021.01/OpenSCAD-2021.01-x86-64.zip"
)]
#[case(
    "openscad",
    "windows",
    "x86",
    "2021.01",
    "https://github.com/openscad/openscad/releases/download/openscad-2021.01/OpenSCAD-2021.01-x86-32.zip"
)]
#[case(
    "openscad",
    "linux",
    "x64",
    "2021.01",
    "https://github.com/openscad/openscad/releases/download/openscad-2021.01/OpenSCAD-2021.01-x86_64.AppImage"
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
    "openscad",
    "windows",
    "x64",
    "2021.01",
    "openscad-2021.01",
    "openscad.com"
)]
fn test_dcc_archive_layout_and_execution_path_agree(
    #[case] provider: &str,
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] root: &str,
    #[case] executable: &str,
) {
    let layout = call(provider, "install_layout", os, arch, version);
    assert_eq!(layout["type"], "archive");
    assert_eq!(layout["strip_prefix"], root);
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
#[case("openscad", "x64", "2021.01", "openscad")]
fn test_dcc_appimages_are_executable_files_not_archives(
    #[case] provider: &str,
    #[case] arch: &str,
    #[case] version: &str,
    #[case] executable: &str,
) {
    let url = call(provider, "download_url", "linux", arch, version);
    assert!(url.as_str().unwrap().ends_with(".AppImage"));
    let layout = call(provider, "install_layout", "linux", arch, version);
    assert_eq!(layout["type"], "binary");
    assert_eq!(layout["target_name"], executable);
    assert_eq!(layout["target_dir"], "bin");
    let path = call(provider, "get_execute_path", "linux", arch, version);
    assert!(path.as_str().unwrap().ends_with("/squashfs-root/AppRun"));
}

#[rstest]
#[case("linux", true)]
#[case("windows", false)]
#[case("macos", false)]
fn test_openscad_extracts_appimages_without_fuse(
    #[case] os: &str,
    #[case] extracts_appimage: bool,
) {
    let (path, content) = source("openscad");
    let mut ctx = ProviderContext::new("openscad", std::env::temp_dir()).with_version("2021.01");
    ctx.platform.os = os.into();
    let install_dir = "/installation with spaces";
    let actions = StarlarkEngine::new()
        .call_function(
            &path,
            &content,
            "post_extract",
            &ctx,
            &[json!("2021.01"), json!(install_dir)],
        )
        .unwrap();
    if extracts_appimage {
        assert_eq!(
            actions,
            json!([
                {"__type": "set_permissions", "path": "bin/openscad", "mode": "755"},
                {
                    "__type": "run_command",
                    "executable": "/installation with spaces/bin/openscad",
                    "args": ["--appimage-extract"],
                    "working_dir": install_dir,
                    "on_failure": "error",
                }
            ])
        );
    } else {
        assert_eq!(actions, json!([]));
    }
}

#[tokio::test]
async fn test_openscad_appimage_cache_requires_extracted_entry_point() {
    let (_directory, ctx) = managed_cache_context();
    let layout = call("openscad", "install_layout", "linux", "x64", "2021.01");
    let runtime = ManifestDrivenRuntime::new("openscad", "openscad", ProviderSource::BuiltIn)
        .with_install_layout(move |_| {
            let layout = layout.clone();
            Box::pin(async move { Ok(Some(layout)) })
        });
    let install_dir = ctx.paths.version_store_dir("openscad", "2021.01");
    std::fs::create_dir_all(install_dir.join("bin")).unwrap();
    std::fs::write(install_dir.join("bin/openscad"), b"AppImage fixture").unwrap();
    assert!(
        !runtime.is_installed("2021.01", &ctx).await.unwrap(),
        "A downloaded AppImage without its extracted launcher must be repaired"
    );
    std::fs::create_dir_all(install_dir.join("squashfs-root")).unwrap();
    std::fs::write(
        install_dir.join("squashfs-root/AppRun"),
        b"launcher fixture",
    )
    .unwrap();
    assert!(runtime.is_installed("2021.01", &ctx).await.unwrap());
}

#[tokio::test]
async fn test_openscad_retains_headless_version_validation() {
    let (path, _) = source("openscad");
    let provider = StarlarkProvider::load(path).await.unwrap();
    let checks = &provider.runtimes()[0].test_commands;
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].command, "{executable} --version");
    assert_eq!(
        checks[0].check_type,
        vx_starlark::provider::types::TestCheckType::Command
    );
    assert!(checks[0].expect_success);
    assert_eq!(
        checks[0].expected_output.as_deref(),
        Some("OpenSCAD version \\d+\\.\\d+")
    );
}

#[rstest]
#[case("openscad", "linux", "arm64", "2021.01")]
#[case("openscad", "macos", "arm64", "2021.01")]
fn test_dcc_unsupported_archives_do_not_fabricate_urls(
    #[case] provider: &str,
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    assert!(call(provider, "download_url", os, arch, version).is_null());
    assert!(call(provider, "install_layout", os, arch, version).is_null());
}

#[rstest]
#[case("openscad", "openscad")]
fn test_dcc_dmg_platforms_use_macos_casks(#[case] provider: &str, #[case] package: &str) {
    let (path, content) = source(provider);
    let install = StarlarkEngine::new()
        .get_variable(&path, &content, "system_install")
        .unwrap()
        .unwrap();
    let cask = install["strategies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|strategy| strategy["manager"] == "brew")
        .unwrap();
    assert_eq!(cask["package"], package);
    assert_eq!(cask["install_args"], "--cask");
    assert_eq!(cask["platforms"], json!(["macos"]));
}
