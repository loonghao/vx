//! material-maker Provider contracts, independent of other optional DCC Providers.

use rstest::rstest;
use serde_json::json;

use vx_starlark::{StarlarkEngine, StarlarkProvider};

#[path = "support/provider_contract.rs"]
mod provider_contract;

use provider_contract::{call, source};

#[rstest]
#[case("material-maker")]
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
    "material-maker",
    "windows",
    "x64",
    "1.7",
    "https://github.com/RodZill4/material-maker/releases/download/1.7/material_maker_1_7_windows.zip"
)]
#[case(
    "material-maker",
    "linux",
    "x64",
    "1.7",
    "https://github.com/RodZill4/material-maker/releases/download/1.7/material_maker_1_7_linux.tar.gz"
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
    "material-maker",
    "windows",
    "x64",
    "1.7",
    "material_maker_1_7_windows",
    "material_maker.exe"
)]
#[case(
    "material-maker",
    "linux",
    "x64",
    "1.7",
    "material_maker_1_7_linux",
    "material_maker.x86_64"
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
#[case("material-maker", "windows", "arm64", "1.7")]
#[case("material-maker", "macos", "x64", "1.7")]
#[case("material-maker", "macos", "arm64", "1.7")]
fn test_dcc_unsupported_archives_do_not_fabricate_urls(
    #[case] provider: &str,
    #[case] os: &str,
    #[case] arch: &str,
    #[case] version: &str,
) {
    assert!(call(provider, "download_url", os, arch, version).is_null());
    assert!(call(provider, "install_layout", os, arch, version).is_null());
}

#[test]
fn test_material_maker_macos_cask_installs_the_discoverable_app_bundle() {
    let (path, content) = source("material-maker");
    let engine = StarlarkEngine::new();
    let install = engine
        .get_variable(&path, &content, "system_install")
        .unwrap()
        .unwrap();
    assert_eq!(
        install["strategies"],
        json!([{
            "manager": "brew",
            "package": "material-maker",
            "priority": 80,
            "install_args": "--cask",
            "platforms": ["macos"]
        }])
    );
    let metadata = vx_starlark::StarMetadata::parse(&content);
    assert!(
        metadata.runtimes[0].system_paths.contains(
            &"/Applications/Material Maker.app/Contents/MacOS/Material Maker".to_string()
        )
    );
    assert!(metadata.runtimes[0].platform_os.is_empty());
    assert!(metadata.platforms.is_none());
}
