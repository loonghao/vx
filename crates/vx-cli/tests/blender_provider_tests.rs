//! blender Provider contracts, independent of other optional DCC Providers.

use rstest::rstest;
use serde_json::json;

use vx_starlark::{StarlarkEngine, StarlarkProvider};

#[path = "support/provider_contract.rs"]
mod provider_contract;

use provider_contract::{call, source};

#[rstest]
#[case("blender")]
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
    "blender",
    "windows",
    "x64",
    "5.2.2",
    "https://download.blender.org/release/Blender5.2/blender-5.2.2-windows-x64.zip"
)]
#[case(
    "blender",
    "windows",
    "arm64",
    "5.2.2",
    "https://download.blender.org/release/Blender5.2/blender-5.2.2-windows-arm64.zip"
)]
#[case(
    "blender",
    "linux",
    "x64",
    "4.5.14",
    "https://download.blender.org/release/Blender4.5/blender-4.5.14-linux-x64.tar.xz"
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
    "blender",
    "windows",
    "x64",
    "5.2.2",
    "blender-5.2.2-windows-x64",
    "blender.exe"
)]
#[case(
    "blender",
    "linux",
    "x64",
    "5.2.2",
    "blender-5.2.2-linux-x64",
    "blender"
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
#[case("blender", "linux", "arm64", "5.2.2")]
#[case("blender", "windows", "x86", "5.2.2")]
#[case("blender", "windows", "arm64", "4.3.0")]
#[case("blender", "macos", "arm64", "5.2.2")]
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
#[case("blender", "blender/blender")]
fn test_dcc_tag_sources_request_only_numeric_release_versions(
    #[case] provider: &str,
    #[case] repository: &str,
) {
    let descriptor = call(provider, "fetch_versions", "windows", "x64", "unused");
    assert_eq!(descriptor["__type"], "fetch_json_versions");
    assert_eq!(descriptor["transform"], "github_tags");
    assert_eq!(descriptor["version_filter"], "numeric");
    assert_eq!(
        descriptor["url"],
        format!("https://api.github.com/repos/{repository}/tags?per_page=100")
    );
}

#[rstest]
#[case("blender", "blender")]
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
