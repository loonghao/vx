//! Provider packaging and runtime contracts.

use std::path::PathBuf;

use serde_json::{Value, json};

use vx_starlark::{ProviderContext, StarlarkEngine};

fn script(name: &str) -> (PathBuf, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(format!("vx-providers/{name}/provider.star"));
    let content = std::fs::read_to_string(&path).unwrap();
    (path, content)
}

fn call(name: &str, os: &str, arch: &str, function: &str, args: &[Value]) -> Value {
    let (path, content) = script(name);
    let mut ctx = ProviderContext::new(name, std::env::temp_dir().join("dcc host with spaces"))
        .with_runtime_name(name);
    ctx.platform.os = os.to_owned();
    ctx.platform.arch = arch.to_owned();
    StarlarkEngine::new()
        .call_function(&path, &content, function, &ctx, args)
        .unwrap()
}

#[rstest::rstest]
#[case("windows", "x64", "Openscreen.Setup.2.0.0.exe")]
#[case("macos", "x64", "Openscreen-Mac-x64-2.0.0.zip")]
#[case("macos", "arm64", "Openscreen-Mac-arm64-2.0.0.zip")]
#[case("linux", "x64", "Openscreen-Linux-2.0.0.AppImage")]
fn test_openscreen_selects_official_distributions(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] asset: &str,
) {
    let url = call("openscreen", os, arch, "download_url", &[json!("2.0.0")]);
    assert_eq!(
        url,
        json!(format!(
            "https://github.com/getopenscreen/openscreen/releases/download/v2.0.0/{asset}"
        ))
    );
}

#[test]
fn test_openscreen_windows_install_is_silent_and_store_scoped() {
    let layout = call(
        "openscreen",
        "windows",
        "x64",
        "install_layout",
        &[json!("2.0.0")],
    );
    assert_eq!(layout["required_paths"], json!(["Openscreen.exe"]));
    let actions = call(
        "openscreen",
        "windows",
        "x64",
        "post_extract",
        &[json!("2.0.0"), json!("C:/vx host/openscreen")],
    );
    assert_eq!(
        actions[0]["executable"],
        "C:/vx host/openscreen/bin/openscreen-installer.exe"
    );
    assert_eq!(actions[0]["__type"], "run_nsis_installer");
    assert_eq!(actions[0]["install_dir"], "C:/vx host/openscreen");
}

#[rstest::rstest]
#[case("openscreen", "linux", "arm64")]
#[case("openscreen", "windows", "arm64")]
fn test_unsupported_distributions_are_not_fabricated(
    #[case] name: &str,
    #[case] os: &str,
    #[case] arch: &str,
) {
    assert!(call(name, os, arch, "download_url", &[json!("2.0.0")]).is_null());
    assert!(call(name, os, arch, "install_layout", &[json!("2.0.0")]).is_null());
}

#[cfg(windows)]
#[rstest::rstest]
#[case("openscreen")]
#[tokio::test]
async fn test_dcc_hook_warm_bootstrap_requires_final_executable(#[case] name: &str) {
    let root = tempfile::tempdir().expect("temporary store");
    let ctx = vx_runtime::RuntimeContext::new(
        std::sync::Arc::new(vx_runtime::RealPathProvider::with_base_dir(root.path())),
        std::sync::Arc::new(vx_runtime::testing::MockHttpClient::new()),
        std::sync::Arc::new(vx_runtime::RealFileSystem),
        std::sync::Arc::new(vx_runtime::testing::MockInstaller::new()),
    );
    let (provider_name, source, bootstrap, final_name) = (
        "openscreen",
        include_str!("../../vx-providers/openscreen/provider.star"),
        "bin/openscreen-installer.exe",
        "Openscreen.exe",
    );
    // The fixture checks the isolated managed store. Keep a real application
    // installed on the test host from satisfying provider system discovery.
    let source = regex::Regex::new(r"(?s)system_paths\s*=\s*\[.*?\]")
        .expect("system path metadata pattern")
        .replace_all(source, "system_paths = []")
        .into_owned();
    let runtime = vx_starlark::build_runtimes(provider_name, source, None::<String>)
        .into_iter()
        .find(|runtime| runtime.name() == name)
        .expect("declared runtime");
    let install_dir = ctx.paths.version_store_dir(name, "2.0.0");
    let bootstrap = install_dir.join(bootstrap);
    std::fs::create_dir_all(bootstrap.parent().expect("bootstrap parent"))
        .expect("bootstrap directory");
    std::fs::write(&bootstrap, b"downloaded bootstrap executable").expect("bootstrap file");

    assert!(
        !runtime
            .is_installed("2.0.0", &ctx)
            .await
            .expect("check incomplete store")
    );
    assert!(
        !runtime
            .is_installed("latest", &ctx)
            .await
            .expect("check incomplete latest")
    );
    assert!(
        runtime
            .get_executable_path_for_version("2.0.0", &ctx)
            .await
            .expect("resolve incomplete store")
            .is_none()
    );

    let final_executable = install_dir.join(final_name);
    std::fs::write(&final_executable, b"final application executable").expect("final executable");
    assert!(
        runtime
            .is_installed("2.0.0", &ctx)
            .await
            .expect("check completed store")
    );
    assert_eq!(
        runtime
            .get_executable_path_for_version("2.0.0", &ctx)
            .await
            .expect("resolve completed store"),
        Some(final_executable)
    );
}

#[test]
fn test_openscreen_rejects_pre_cli_releases() {
    assert!(
        call(
            "openscreen",
            "windows",
            "x64",
            "download_url",
            &[json!("1.0.0")]
        )
        .is_null()
    );
    assert!(
        call(
            "openscreen",
            "windows",
            "x64",
            "install_layout",
            &[json!("1.0.0")]
        )
        .is_null()
    );
}

#[test]
fn test_openscreen_linux_extracts_appimage_before_exposing_the_application() {
    let layout = call(
        "openscreen",
        "linux",
        "x64",
        "install_layout",
        &[json!("2.0.0")],
    );
    assert_eq!(layout["target_dir"], "bin");
    assert_eq!(layout["executable_paths"], json!(["bin/openscreen"]));
    assert_eq!(layout["required_paths"], json!(["squashfs-root/AppRun"]));
    let actions = call(
        "openscreen",
        "linux",
        "x64",
        "post_extract",
        &[json!("2.0.0"), json!("/vx host/openscreen")],
    );
    assert_eq!(
        actions[0]["executable"],
        "/vx host/openscreen/bin/openscreen"
    );
    assert_eq!(actions[0]["args"], json!(["--appimage-extract"]));
    assert_eq!(actions[0]["working_dir"], "/vx host/openscreen");
    assert_eq!(actions[0]["on_failure"], "error");
    let executable = call(
        "openscreen",
        "linux",
        "x64",
        "get_execute_path",
        &[json!("2.0.0")],
    );
    assert!(
        executable
            .as_str()
            .unwrap()
            .ends_with("/squashfs-root/AppRun")
    );
}
