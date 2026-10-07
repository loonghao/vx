//! freecad Provider contracts, independent of other optional DCC Providers.

use rstest::rstest;
use serde_json::json;

use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime};
use vx_starlark::{StarlarkEngine, StarlarkProvider};

#[path = "support/provider_contract.rs"]
mod provider_contract;

#[path = "support/managed_cache.rs"]
mod managed_cache;

use managed_cache::managed_cache_context;
use provider_contract::{call, call_runtime, source};

#[rstest]
#[case("freecad")]
#[tokio::test]
async fn test_load_dcc_provider_with_system_discovery(#[case] name: &str) {
    let (path, _) = source(name);
    let provider = StarlarkProvider::load(&path).await.unwrap();
    assert_eq!(provider.name(), name);
    assert_eq!(provider.runtimes().len(), 2);
    assert!(
        !provider.runtimes()[0].system_paths.is_empty(),
        "{name} must discover existing DCC-MCP application installations"
    );
}

#[rstest]
#[case("freecadcmd")]
#[case("FreeCADCmd")]
#[case("freecad-cmd")]
fn test_freecad_console_runtime_never_resolves_to_gui_executable(#[case] runtime: &str) {
    let path = call_runtime(
        "freecad",
        "get_execute_path",
        "windows",
        "x64",
        "1.1.4",
        Some(runtime),
    );
    assert!(path.as_str().unwrap().ends_with("/bin/FreeCADCmd.exe"));
    let linux = call_runtime(
        "freecad",
        "get_execute_path",
        "linux",
        "x64",
        "1.1.4",
        Some(runtime),
    );
    assert!(
        linux.is_null(),
        "The Linux AppImage does not expose a standalone FreeCADCmd"
    );
}

#[test]
fn test_freecad_console_runtime_is_bundled_and_windows_only() {
    let (path, content) = source("freecad");
    let runtimes = StarlarkEngine::new()
        .get_variable(&path, &content, "runtimes")
        .unwrap()
        .unwrap();
    assert_eq!(
        runtimes[0]["test_commands"],
        json!([{"command": "{executable}", "check_type": "check_file", "name": "installed_application_executable"}])
    );
    let console = runtimes
        .as_array()
        .unwrap()
        .iter()
        .find(|runtime| runtime["name"] == "freecadcmd")
        .unwrap();
    assert_eq!(console["bundled_with"], "freecad");
    assert_eq!(console["executable"], "FreeCADCmd");
    assert_eq!(console["aliases"], json!(["FreeCADCmd", "freecad-cmd"]));
    assert_eq!(console["platform_constraint"], json!({"os": ["windows"]}));

    // CLI embedding and runtime construction use static metadata rather than
    // executing provider.star; both paths must see the bundled entry point.
    let metadata = vx_starlark::StarMetadata::parse(&content);
    let console = metadata
        .runtimes
        .iter()
        .find(|runtime| runtime.name.as_deref() == Some("freecadcmd"))
        .unwrap();
    assert_eq!(console.bundled_with.as_deref(), Some("freecad"));
    assert_eq!(console.aliases, ["FreeCADCmd", "freecad-cmd"]);
    assert_eq!(
        console.system_paths,
        ["C:/Program Files/FreeCAD*/bin/FreeCADCmd.exe"]
    );
    assert_eq!(console.platform_os, ["windows"]);
    let runtimes = vx_starlark::build_runtimes("freecad", content, Some("freecad"));
    let console = runtimes
        .iter()
        .find(|runtime| runtime.name() == "freecadcmd")
        .unwrap();
    assert_eq!(console.store_name(), "freecad");
    assert!(!console.is_version_installable("1.1.4"));
    assert_eq!(console.aliases(), ["FreeCADCmd", "freecad-cmd"]);
}

#[rstest]
#[case("freecad", "FreeCAD")]
#[case("freecadcmd", "FreeCADCmd")]
#[tokio::test]
async fn test_freecad_managed_cache_requires_gui_and_console_binaries(
    #[case] runtime_name: &str,
    #[case] executable: &str,
) {
    let (_directory, ctx) = managed_cache_context();
    let layout = call("freecad", "install_layout", "windows", "x64", "1.1.4");
    assert_eq!(
        layout["required_paths"],
        json!(["bin/FreeCAD.exe", "bin/FreeCADCmd.exe"])
    );
    let mut runtime = ManifestDrivenRuntime::new(runtime_name, "freecad", ProviderSource::BuiltIn)
        .with_executable(executable)
        .with_install_layout(move |_| {
            let layout = layout.clone();
            Box::pin(async move { Ok(Some(layout)) })
        });
    if runtime_name == "freecadcmd" {
        runtime = runtime.with_bundled_with("freecad");
    }
    let install_dir = ctx.paths.version_store_dir("freecad", "1.1.4");
    std::fs::create_dir_all(install_dir.join("bin")).unwrap();
    std::fs::write(install_dir.join("bin/FreeCAD.exe"), b"").unwrap();
    assert!(
        !runtime.is_installed("1.1.4", &ctx).await.unwrap(),
        "A GUI-only cache must not satisfy either FreeCAD runtime"
    );
    std::fs::write(install_dir.join("bin/FreeCADCmd.exe"), b"").unwrap();
    assert!(runtime.is_installed("1.1.4", &ctx).await.unwrap());
    std::fs::remove_file(install_dir.join("bin/FreeCAD.exe")).unwrap();
    assert!(
        !runtime.is_installed("1.1.4", &ctx).await.unwrap(),
        "A console-only cache must also be repaired"
    );
}

#[test]
fn test_freecad_dcc_environment_points_to_console_only_on_windows() {
    let windows = call("freecad", "environment", "windows", "x64", "1.1.4");
    let executable = windows
        .as_array()
        .unwrap()
        .iter()
        .find(|op| op["key"] == "DCC_MCP_FREECAD_EXECUTABLE")
        .unwrap();
    assert_eq!(executable["op"], "set");
    assert!(
        executable["value"]
            .as_str()
            .unwrap()
            .ends_with("/bin/FreeCADCmd.exe")
    );
    let linux = call("freecad", "environment", "linux", "x64", "1.1.4");
    assert!(
        linux
            .as_array()
            .unwrap()
            .iter()
            .all(|op| op["key"] != "DCC_MCP_FREECAD_EXECUTABLE")
    );
}

#[rstest]
#[case(
    "freecad",
    "windows",
    "x64",
    "1.1.4",
    "https://github.com/FreeCAD/FreeCAD/releases/download/1.1.4/FreeCAD_1.1.4-Windows-x86_64-py311.7z"
)]
#[case(
    "freecad",
    "windows",
    "x64",
    "1.0.2",
    "https://github.com/FreeCAD/FreeCAD/releases/download/1.0.2/FreeCAD_1.0.2-conda-Windows-x86_64-py311.7z"
)]
#[case(
    "freecad",
    "linux",
    "arm64",
    "1.1.4",
    "https://github.com/FreeCAD/FreeCAD/releases/download/1.1.4/FreeCAD_1.1.4-Linux-aarch64-py311.AppImage"
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
    "freecad",
    "windows",
    "x64",
    "1.1.4",
    "FreeCAD_1.1.4-Windows-x86_64-py311",
    "bin/FreeCAD.exe"
)]
#[case(
    "freecad",
    "windows",
    "x64",
    "1.0.2",
    "FreeCAD_1.0.2-conda-Windows-x86_64-py311",
    "bin/FreeCAD.exe"
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
#[case("freecad", "arm64", "1.1.4", "freecad")]
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
#[case("freecad", "windows", "arm64", "1.1.4")]
#[case("freecad", "windows", "x64", "0.21.2")]
#[case("freecad", "macos", "arm64", "1.1.4")]
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
#[case("freecad", "freecad")]
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
