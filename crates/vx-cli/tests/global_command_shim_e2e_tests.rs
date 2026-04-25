//! E2E tests for direct global command execution via stacked shims.

mod common;

use common::{
    assert_output_contains, assert_success, init_test_env, network_tests_enabled, stdout_str,
    vx_available, vx_binary,
};
use rstest::rstest;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

#[rstest]
#[test]
fn test_where_and_direct_vite_help_after_npm_global_install() {
    init_test_env();
    if !vx_available() || !network_tests_enabled() {
        return;
    }

    let test_exe = std::env::current_exe().expect("failed to resolve current test executable");
    let debug_dir = test_exe
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .expect("failed to resolve target/debug directory");
    let vx_path = debug_dir.join(if cfg!(windows) { "vx.exe" } else { "vx" });
    assert!(
        vx_path.exists(),
        "expected vx binary at {}",
        vx_path.display()
    );
    #[allow(clippy::disallowed_methods)]
    unsafe {
        std::env::set_var("VX_BINARY", &vx_path);
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let vx_home_str = vx_home
        .to_str()
        .expect("temp path should be valid UTF-8")
        .to_string();

    let install_output =
        run_vx_for_test(&vx_path, temp.path(), &vx_home_str, &["npm", "install", "-g", "vite"])
            .expect("failed to run vx npm install -g vite");
    assert_success(&install_output, "vx npm install -g vite");

    let where_output = run_vx_for_test(&vx_path, temp.path(), &vx_home_str, &["where", "vite"])
        .expect("failed to run vx where vite");
    assert_success(&where_output, "vx where vite");
    assert!(
        !stdout_str(&where_output).trim().is_empty(),
        "vx where vite should return a non-empty path"
    );

    let vx_bin_dir = vx_binary()
        .parent()
        .expect("vx binary should have parent directory")
        .to_path_buf();
    let vite_shim = if cfg!(windows) {
        vx_bin_dir.join("vite.cmd")
    } else {
        vx_bin_dir.join("vite")
    };

    assert!(
        vite_shim.exists(),
        "direct shim should exist in vx bin dir: {}",
        vite_shim.display()
    );

    let help_output = Command::new(&vite_shim)
        .arg("--help")
        .env("VX_HOME", vx_home_str)
        .output()
        .expect("failed to run direct vite --help");
    assert_success(&help_output, "direct vite --help");
    assert_output_contains(&help_output, "vite", "direct vite --help output");
}

fn run_vx_for_test(
    vx_path: &std::path::Path,
    cwd: &std::path::Path,
    vx_home: &str,
    args: &[&str],
) -> std::io::Result<Output> {
    Command::new(vx_path)
        .args(args)
        .current_dir(cwd)
        .env("VX_HOME", vx_home)
        .output()
}
