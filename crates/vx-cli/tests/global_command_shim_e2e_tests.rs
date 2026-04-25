//! E2E tests for direct global command execution via stacked shims.

mod common;

use common::{
    assert_output_contains, assert_success, init_test_env, network_tests_enabled, stdout_str,
    vx_available, vx_binary,
};
use rstest::rstest;
use std::process::{Command, Output};
use tempfile::TempDir;

#[rstest]
#[test]
fn test_where_and_direct_vite_help_after_npm_global_install() {
    init_test_env();
    if !vx_available() || !network_tests_enabled() {
        return;
    }

    let vx_path = vx_binary();
    if !vx_path.exists() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let vx_home_str = vx_home
        .to_str()
        .expect("temp path should be valid UTF-8")
        .to_string();

    let install_output = run_vx_for_test(
        &vx_path,
        temp.path(),
        &vx_home_str,
        &["npm", "install", "-g", "vite"],
    )
    .expect("failed to run vx npm install -g vite");
    assert_success(&install_output, "vx npm install -g vite");

    let where_output = run_vx_for_test(&vx_path, temp.path(), &vx_home_str, &["where", "vite"])
        .expect("failed to run vx where vite");
    assert_success(&where_output, "vx where vite");
    assert!(
        !stdout_str(&where_output).trim().is_empty(),
        "vx where vite should return a non-empty path"
    );

    let shim_name = if cfg!(windows) { "vite.cmd" } else { "vite" };
    let mut shim_candidates = Vec::new();
    if let Some(vx_bin_dir) = vx_path.parent() {
        shim_candidates.push(vx_bin_dir.join(shim_name));
    }
    shim_candidates.push(vx_home.join("shims").join(shim_name));

    let vite_shim = shim_candidates
        .iter()
        .find(|path| path.exists())
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "direct shim should exist in one of: {}",
                shim_candidates
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        });

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
