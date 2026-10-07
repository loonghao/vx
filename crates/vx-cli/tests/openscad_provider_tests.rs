//! openscad Provider contracts, independent of other optional DCC Providers.

use rstest::rstest;
use serde_json::json;

use vx_starlark::{StarlarkEngine, StarlarkProvider};

#[path = "support/provider_contract.rs"]
mod provider_contract;

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
    assert!(
        path.as_str()
            .unwrap()
            .ends_with(&format!("/bin/{executable}"))
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
