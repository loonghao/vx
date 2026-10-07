//! Exercise post-extract descriptors through the Starlark/runtime bridge.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use rstest::rstest;
use serde_json::{Value, json};

use vx_runtime::{
    RealFileSystem, RealPathProvider, Runtime, RuntimeContext,
    testing::{MockHttpClient, MockInstaller},
};
use vx_starlark::build_runtimes;

const OUTPUT_ENV: &str = "VX_DCC_HOOK_CHILD_OUTPUT";
const CHILD_TEST: &str = "test_dcc_hook_child_records_args";

// The copied test executable is a real native argv recorder, so these tests
// need neither an installed Python runtime nor a shell-based mock executable.
#[test]
fn test_dcc_hook_child_records_args() {
    let Some(output) = std::env::var_os(OUTPUT_ENV) else {
        return;
    };
    let arguments: Vec<String> = std::env::args()
        .skip_while(|arg| arg != "--")
        .skip(1)
        .collect();
    std::fs::write(
        output,
        serde_json::to_vec(&json!({
            "arguments": arguments,
            "working_dir": std::env::current_dir().expect("child working directory"),
        }))
        .expect("serialize child result"),
    )
    .expect("record child arguments");
}

fn context(root: &Path) -> RuntimeContext {
    RuntimeContext::new(
        Arc::new(RealPathProvider::with_base_dir(root)),
        Arc::new(MockHttpClient::new()),
        Arc::new(RealFileSystem),
        Arc::new(MockInstaller::new()),
    )
}

fn runtime(actions: &str) -> Arc<dyn Runtime> {
    let content = format!(
        r#"
load("@vx//stdlib:install.star", "create_shim", "run_command", "run_nsis_installer")
name = "dcc-hook-test"
description = "Post-extract descriptor contract test"
runtimes = [{{"name": name, "executable": name}}]

def post_extract(ctx, version, install_dir):
    return [{actions}]
"#
    );
    build_runtimes("dcc-hook-test", content, None::<String>)
        .into_iter()
        .next()
        .expect("runtime")
}

fn native_child(install_dir: &Path) -> PathBuf {
    std::fs::create_dir_all(install_dir).expect("create install directory");
    let target = install_dir.join(if cfg!(windows) {
        "target executable.exe"
    } else {
        "target executable"
    });
    std::fs::copy(std::env::current_exe().expect("test executable"), &target)
        .expect("copy native argv recorder");
    target
}

fn child_arguments() -> Vec<String> {
    ["--exact", CHILD_TEST, "--nocapture", "--"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn starlark_string(value: impl AsRef<str>) -> String {
    serde_json::to_string(value.as_ref()).expect("quote Starlark string")
}

fn read_result(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).expect("child output exists"))
        .expect("child output is JSON")
}

#[rstest]
#[case(false, false)]
#[case(false, true)]
#[case(true, true)]
#[tokio::test]
async fn test_dcc_hook_shim_preserves_native_arguments(
    #[case] absolute_target: bool,
    #[case] explicit_directory: bool,
) {
    let root = tempfile::tempdir().expect("temporary root");
    let ctx = context(&root.path().join("store with spaces"));
    let install_dir = ctx.paths.version_store_dir("dcc-hook-test", "1.0");
    let child = native_child(&install_dir);
    let target = if absolute_target {
        child.clone()
    } else {
        PathBuf::from(child.file_name().expect("child filename"))
    };
    let prefix = [
        "prefix with spaces",
        "literal & %PATH% $(echo injected)",
        "double\"quote",
        "trailing slash\\",
        "",
    ];
    let mut args = child_arguments();
    args.extend(prefix.iter().map(|value| value.to_string()));
    let directory_argument = if explicit_directory {
        ", shim_dir = \"launch directory\""
    } else {
        ""
    };
    let hook = runtime(&format!(
        "create_shim(\"dcc-launch\", {}, args = {}{})",
        starlark_string(target.to_string_lossy()),
        serde_json::to_string(&args).expect("serialize prefix arguments"),
        directory_argument,
    ));
    hook.post_install("1.0", &ctx)
        .await
        .expect("create hook shim");

    let output_file = root.path().join("child result.json");
    let directory = if explicit_directory {
        install_dir.join("launch directory")
    } else {
        install_dir
    };
    let shim = directory.join(if cfg!(windows) {
        "dcc-launch.cmd"
    } else {
        "dcc-launch"
    });
    // Rust's Windows process implementation wraps .cmd files in cmd.exe with
    // the extra outer quoting needed when the script path contains spaces.
    let mut command = Command::new(&shim);
    let status = command
        .arg("user argument with spaces")
        .env(OUTPUT_ENV, &output_file)
        .status()
        .expect("execute generated shim");
    assert!(status.success(), "shim failed with {status}");
    let mut expected: Vec<&str> = prefix.into_iter().collect();
    expected.push("user argument with spaces");
    assert_eq!(read_result(&output_file)["arguments"], json!(expected));
}

#[rstest]
#[case(false)]
#[case(true)]
#[tokio::test]
async fn test_dcc_hook_run_command_preserves_working_directory(#[case] absolute_directory: bool) {
    let root = tempfile::tempdir().expect("temporary root");
    let ctx = context(root.path());
    let install_dir = ctx.paths.version_store_dir("dcc-hook-test", "1.0");
    let child = native_child(&install_dir);
    let working_dir = install_dir.join("working directory with spaces");
    std::fs::create_dir_all(&working_dir).expect("working directory");
    let descriptor_dir = if absolute_directory {
        working_dir.clone()
    } else {
        PathBuf::from("working directory with spaces")
    };
    let output = root.path().join("working directory.json");
    let hook = runtime(&format!(
        "run_command({}, args = {}, working_dir = {}, env = {{{}: {}}}, on_failure = \"error\")",
        starlark_string(child.to_string_lossy()),
        serde_json::to_string(&child_arguments()).expect("serialize child arguments"),
        starlark_string(descriptor_dir.to_string_lossy()),
        starlark_string(OUTPUT_ENV),
        starlark_string(output.to_string_lossy()),
    ));
    hook.post_install("1.0", &ctx)
        .await
        .expect("execute hook command");
    assert_eq!(
        std::fs::canonicalize(PathBuf::from(
            read_result(&output)["working_dir"]
                .as_str()
                .expect("recorded directory")
        ))
        .expect("canonical child directory"),
        std::fs::canonicalize(working_dir).expect("canonical expected directory")
    );
}

#[tokio::test]
async fn test_dcc_hook_command_error_aborts_installation() {
    let root = tempfile::tempdir().expect("temporary root");
    let ctx = context(root.path());
    std::fs::create_dir_all(ctx.paths.version_store_dir("dcc-hook-test", "1.0"))
        .expect("install directory");
    let hook = runtime("run_command(\"vx-missing-dcc-hook-command\", [], on_failure = \"error\")");
    let error = hook
        .post_install("1.0", &ctx)
        .await
        .expect_err("required hook must fail");
    assert!(error.to_string().contains("failed to run"));
}

#[tokio::test]
async fn test_dcc_hook_shim_write_error_aborts_installation() {
    let root = tempfile::tempdir().expect("temporary root");
    let ctx = context(root.path());
    let install_dir = ctx.paths.version_store_dir("dcc-hook-test", "1.0");
    std::fs::create_dir_all(&install_dir).expect("install directory");
    std::fs::write(install_dir.join("blocked"), b"a file is not a directory")
        .expect("block shim directory");
    let hook = runtime("create_shim(\"dcc-launch\", \"target\", shim_dir = \"blocked\")");
    hook.post_install("1.0", &ctx)
        .await
        .expect_err("shim write error must fail");
}

#[cfg(windows)]
fn fake_nsis_installer() -> PathBuf {
    use std::sync::OnceLock;

    static FIXTURE: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let root = tempfile::tempdir().expect("fake NSIS build directory");
        let source = root.path().join("fake_nsis.rs");
        let executable = root.path().join("fake NSIS installer.exe");
        // The existing Rust test toolchain builds a dependency-free native
        // fixture. Reading GetCommandLineW models the NSIS parser directly.
        std::fs::write(&source, r#"
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCommandLineW() -> *const u16;
}

fn main() {
    let command_line = unsafe {
        let pointer = GetCommandLineW();
        let mut length = 0;
        while *pointer.add(length) != 0 {
            length += 1;
        }
        String::from_utf16(std::slice::from_raw_parts(pointer, length)).expect("UTF-16 command line")
    };
    let (prefix, destination) = command_line.rsplit_once(" /D=").expect("final /D= destination");
    assert!(prefix.ends_with(" /S /currentuser"), "fixed NSIS options: {prefix}");
    assert!(!destination.contains('"'), "NSIS destination must be unquoted");
    assert!(std::path::Path::new(destination).is_absolute(), "absolute NSIS destination");
    std::fs::create_dir_all(destination).expect("create requested NSIS destination");
    std::fs::write(std::path::Path::new(destination).join("nsis-command-line.txt"), &command_line)
        .expect("record raw Windows command line");
    if destination.ends_with("exit failure") {
        std::process::exit(23);
    }
}
"#).expect("write fake NSIS source");
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let output = Command::new(compiler)
            .args(["--edition=2024", "--crate-name", "vx_nsis_fixture"])
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .output()
            .expect("compile fake NSIS with the existing Rust test toolchain");
        assert!(output.status.success(), "fake NSIS compilation failed: {}", String::from_utf8_lossy(&output.stderr));
        (root, executable)
    }).1.clone()
}

#[cfg(windows)]
#[tokio::test]
async fn test_dcc_hook_nsis_preserves_unquoted_final_destination_with_spaces() {
    let root = tempfile::tempdir().expect("temporary NSIS store");
    let ctx = context(root.path());
    let destination = root
        .path()
        .join("NSIS destination 'with spaces' & literal % 中文");
    let executable = fake_nsis_installer();
    let hook = runtime(&format!(
        "run_nsis_installer({}, {})",
        starlark_string(executable.to_string_lossy()),
        starlark_string(destination.to_string_lossy()),
    ));

    hook.post_install("1.0", &ctx)
        .await
        .expect("run fake NSIS installer");

    let command_line = std::fs::read_to_string(destination.join("nsis-command-line.txt"))
        .expect("native NSIS fixture wrote to the exact requested directory");
    assert!(
        command_line.ends_with(&format!(" /S /currentuser /D={}", destination.display())),
        "{command_line}"
    );
}

#[cfg(windows)]
#[tokio::test]
async fn test_dcc_hook_nsis_failure_stops_installation() {
    let root = tempfile::tempdir().expect("temporary NSIS store");
    let ctx = context(root.path());
    let destination = root.path().join("exit failure");
    let hook = runtime(&format!(
        "run_nsis_installer({}, {})",
        starlark_string(fake_nsis_installer().to_string_lossy()),
        starlark_string(destination.to_string_lossy()),
    ));

    let error = hook
        .post_install("1.0", &ctx)
        .await
        .expect_err("NSIS exit failure must stop installation");
    assert!(
        error.to_string().contains("NSIS installer exited"),
        "{error}"
    );
}

#[cfg(windows)]
#[rstest]
#[case("relative/destination")]
#[case("C:/invalid\"destination")]
#[case("C:/invalid\0destination")]
#[case("C:/invalid\rdestination")]
#[case("C:/invalid\ndestination")]
#[tokio::test]
async fn test_dcc_hook_nsis_rejects_invalid_destination_before_launch(#[case] destination: &str) {
    let root = tempfile::tempdir().expect("temporary NSIS store");
    let ctx = context(root.path());
    let hook = runtime(&format!(
        "run_nsis_installer({}, {})",
        starlark_string(root.path().join("missing-installer.exe").to_string_lossy()),
        starlark_string(destination),
    ));

    let error = hook
        .post_install("1.0", &ctx)
        .await
        .expect_err("invalid NSIS destination must fail before process creation");
    assert!(
        error
            .to_string()
            .contains("NSIS install_dir must be absolute"),
        "{error}"
    );
}

#[cfg(not(windows))]
#[tokio::test]
async fn test_dcc_hook_nsis_reports_unsupported_platform() {
    let root = tempfile::tempdir().expect("temporary NSIS store");
    let ctx = context(root.path());
    let hook = runtime("run_nsis_installer(\"/tmp/installer.exe\", \"/tmp/destination\")");

    let error = hook
        .post_install("1.0", &ctx)
        .await
        .expect_err("NSIS is Windows-only");
    assert!(
        error.to_string().contains("supported only on Windows"),
        "{error}"
    );
}
