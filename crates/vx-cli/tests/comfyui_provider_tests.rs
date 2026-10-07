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
#[case("comfyui", "nvidia")]
#[case("comfyui-amd", "amd")]
#[case("comfyui-intel", "intel")]
#[tokio::test]
async fn test_comfyui_runtime_bridge_selects_the_requested_backend(
    #[case] runtime_name: &str,
    #[case] backend: &str,
) {
    let (_, content) = script("comfyui");
    let runtimes = vx_starlark::build_runtimes("comfyui", content, None::<String>);
    let runtime = runtimes
        .iter()
        .find(|runtime| runtime.name() == runtime_name)
        .unwrap();
    let platform = vx_runtime::Platform::new(vx_runtime::Os::Windows, vx_runtime::Arch::X86_64);
    let url = runtime
        .download_url("0.39.0", &platform)
        .await
        .unwrap()
        .unwrap();
    assert!(url.ends_with(&format!("/ComfyUI_windows_portable_{backend}.7z")));
}

#[test]
fn test_comfyui_uses_embedded_python_and_a_store_local_script() {
    let url = call(
        "comfyui",
        "windows",
        "x64",
        "download_url",
        &[json!("0.39.0")],
    );
    assert!(
        url.as_str()
            .unwrap()
            .ends_with("/v0.39.0/ComfyUI_windows_portable_nvidia.7z")
    );
    let layout = call(
        "comfyui",
        "windows",
        "x64",
        "install_layout",
        &[json!("0.39.0")],
    );
    assert_eq!(
        layout["executable_paths"],
        json!(["python_embeded/python.exe"])
    );
    assert_eq!(
        layout["required_paths"],
        json!(["ComfyUI/main.py", "comfyui.cmd"])
    );
    let actions = call(
        "comfyui",
        "windows",
        "x64",
        "post_extract",
        &[json!("0.39.0"), json!("C:/vx host/comfyui")],
    );
    assert_eq!(actions[0]["__type"], "create_shim");
    assert_eq!(
        actions[0]["target"],
        "C:/vx host/comfyui/python_embeded/python.exe"
    );
    assert_eq!(actions[0]["args"][1], "C:/vx host/comfyui/ComfyUI/main.py");
    assert_eq!(actions[0]["shim_dir"], "C:/vx host/comfyui");
}

#[rstest::rstest]
#[case("comfyui", "linux", "x64")]
#[case("comfyui", "windows", "arm64")]
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
#[case("comfyui")]
#[case("comfyui-amd")]
#[case("comfyui-intel")]
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
        "comfyui",
        include_str!("../../vx-providers/comfyui/provider.star"),
        "python_embeded/python.exe",
        "comfyui.cmd",
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
    let install_dir = ctx.paths.version_store_dir(name, "1.0");
    let bootstrap = install_dir.join(bootstrap);
    std::fs::create_dir_all(bootstrap.parent().expect("bootstrap parent"))
        .expect("bootstrap directory");
    std::fs::write(&bootstrap, b"downloaded bootstrap executable").expect("bootstrap file");
    if name != "openscreen" {
        std::fs::create_dir_all(install_dir.join("ComfyUI")).expect("application directory");
        std::fs::write(
            install_dir.join("ComfyUI/main.py"),
            b"application entrypoint",
        )
        .expect("application script");
    }

    assert!(
        !runtime
            .is_installed("1.0", &ctx)
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
            .get_executable_path_for_version("1.0", &ctx)
            .await
            .expect("resolve incomplete store")
            .is_none()
    );

    let final_executable = install_dir.join(final_name);
    std::fs::write(&final_executable, b"final application executable").expect("final executable");
    assert!(
        runtime
            .is_installed("1.0", &ctx)
            .await
            .expect("check completed store")
    );
    assert_eq!(
        runtime
            .get_executable_path_for_version("1.0", &ctx)
            .await
            .expect("resolve completed store"),
        Some(final_executable)
    );
}
