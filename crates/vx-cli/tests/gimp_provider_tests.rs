//! Provider packaging and runtime contracts.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Value, json};

use vx_runtime::Runtime;
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

fn runtime_with_system_root(root: &Path) -> Arc<dyn Runtime> {
    let (_, source) = script("gimp");
    let prefix = format!("{}/", root.to_string_lossy().replace('\\', "/"));
    // Exercise the actual provider's glob discovery inside an isolated drive.
    let source = source.replace("C:/", &prefix);
    vx_starlark::build_runtimes("gimp", source, None::<String>)
        .into_iter()
        .find(|runtime| runtime.name() == "gimp")
        .expect("GIMP runtime")
}

#[test]
fn test_gimp_system_paths_match_cli_static_metadata() {
    let (path, content) = script("gimp");
    let evaluated = StarlarkEngine::new()
        .get_variable(&path, &content, "runtimes")
        .unwrap()
        .unwrap();
    let metadata = vx_starlark::StarMetadata::parse(&content);
    let runtime = metadata
        .runtimes
        .iter()
        .find(|runtime| runtime.name.as_deref() == Some("gimp"))
        .expect("GIMP CLI metadata");

    assert_eq!(runtime.system_paths.len(), 28);
    assert_eq!(json!(runtime.system_paths), evaluated[0]["system_paths"]);
}

#[rstest::rstest]
#[case("Program Files/GIMP 3/bin/gimp-console-3.exe")]
#[case("Program Files/GIMP 3/bin/gimp-3.exe")]
#[case("Program Files/GIMP 3/bin/gimp-console-3.2.exe")]
#[case("Program Files/GIMP 3/bin/gimp-3.2.exe")]
#[case("Program Files/GIMP 3/bin/gimp-console-3.0.exe")]
#[case("Program Files/GIMP 3/bin/gimp-3.0.exe")]
#[case("Users/gimp user/AppData/Local/Programs/GIMP 3/bin/gimp-console-3.exe")]
#[case("Users/gimp user/AppData/Local/Programs/GIMP 3/bin/gimp-3.exe")]
#[case("Users/gimp user/AppData/Local/Programs/GIMP 3/bin/gimp-console-3.2.exe")]
#[case("Users/gimp user/AppData/Local/Programs/GIMP 3/bin/gimp-3.2.exe")]
#[case("Users/gimp user/AppData/Local/Programs/GIMP 3/bin/gimp-console-3.0.exe")]
#[case("Users/gimp user/AppData/Local/Programs/GIMP 3/bin/gimp-3.0.exe")]
#[case("Program Files/GIMP 2/bin/gimp-console-2.10.exe")]
#[case("Program Files/GIMP 2/bin/gimp-2.10.exe")]
#[case("Program Files (x86)/GIMP 2/bin/gimp-console-2.10.exe")]
#[case("Program Files (x86)/GIMP 2/bin/gimp-2.10.exe")]
#[case("Users/gimp user/AppData/Local/Programs/GIMP 2/bin/gimp-console-2.10.exe")]
#[case("Users/gimp user/AppData/Local/Programs/GIMP 2/bin/gimp-2.10.exe")]
#[tokio::test]
async fn test_gimp_discovers_windows_installed_executables(#[case] relative_path: &str) {
    let root = tempfile::tempdir().expect("isolated Windows installation roots");
    let executable = root.path().join(relative_path);
    std::fs::create_dir_all(executable.parent().expect("application bin directory"))
        .expect("application directory");
    std::fs::write(&executable, b"installed GIMP executable").expect("application executable");
    let runtime = runtime_with_system_root(root.path());
    let prep = runtime
        .prepare_execution("system", &vx_runtime::testing::mock_execution_context())
        .await
        .expect("system application discovery");

    assert_eq!(
        prep.executable_override
            .expect("discovered executable")
            .canonicalize()
            .unwrap(),
        executable.canonicalize().unwrap()
    );
    assert!(!prep.use_system_path);
}

#[tokio::test]
async fn test_gimp_prefers_current_console_alias_over_older_or_gui_executables() {
    let root = tempfile::tempdir().expect("isolated Windows installation roots");
    let bin = root.path().join("Program Files/GIMP 3/bin");
    std::fs::create_dir_all(&bin).expect("application directory");
    for executable in ["gimp-3.exe", "gimp-console-3.0.exe", "gimp-console-3.exe"] {
        std::fs::write(bin.join(executable), b"installed GIMP executable")
            .expect("application executable");
    }
    let prep = runtime_with_system_root(root.path())
        .prepare_execution("system", &vx_runtime::testing::mock_execution_context())
        .await
        .expect("system application discovery");

    assert_eq!(
        prep.executable_override
            .expect("preferred console executable")
            .canonicalize()
            .unwrap(),
        bin.join("gimp-console-3.exe").canonicalize().unwrap()
    );
}

#[test]
fn test_gimp_system_manager_routes_are_platform_scoped() {
    let (_, content) = script("gimp");
    // A downloaded installer is never treated as an already installed editor.
    assert!(call("gimp", "windows", "x64", "download_url", &[json!("latest")]).is_null());
    assert!(content.contains("GIMP.GIMP.3"));
    assert!(content.contains("platforms = [\"macos\"]"));
}

#[rstest::rstest]
#[case("windows")]
#[case("linux")]
#[case("macos")]
fn test_gimp_latest_resolves_a_system_package_without_an_upstream_pin(#[case] os: &str) {
    let versions = call("gimp", os, "x64", "fetch_versions", &[]);
    assert_eq!(
        versions,
        json!([{"version": "system", "lts": true, "prerelease": false}])
    );
}

#[test]
fn test_gimp_supports_windows_package_managers_without_winget() {
    let (path, content) = script("gimp");
    let descriptor = StarlarkEngine::new()
        .get_variable(&path, &content, "system_install")
        .unwrap()
        .unwrap();
    let strategies = descriptor["strategies"].as_array().unwrap();
    assert!(strategies.iter().any(|strategy| {
        strategy["manager"] == "choco"
            && strategy["package"] == "gimp"
            && strategy["platforms"] == json!(["windows"])
    }));
}
