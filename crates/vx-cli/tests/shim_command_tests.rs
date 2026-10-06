//! Integration coverage for `vx shim` (RFC 0042).
//!
//! These tests drive the real `vx` binary against a temporary `VX_HOME` and
//! write shims into a temporary directory, so they never touch the developer's
//! PATH or the `~/.local/bin` wrapper a user may already have.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{assert_success, init_test_env, vx_available, vx_binary};
use tempfile::TempDir;
use vx_runtime::{ShimRegistry, VX_SHIM_MARKER};

fn run_vx(vx_home: &Path, args: &[&str]) -> std::io::Result<Output> {
    Command::new(vx_binary())
        .args(args)
        .env("VX_HOME", vx_home)
        .output()
}

/// Every file a shim produces in `dir` for the current platform
fn expected_files(dir: &Path, name: &str) -> Vec<PathBuf> {
    let mut files = vec![dir.join(name)];
    if cfg!(windows) {
        files.push(dir.join(format!("{}.cmd", name)));
    }
    files
}

#[test]
fn test_shim_add_creates_a_platform_shim() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let shim_dir = temp.path().join("shims");

    let output = run_vx(
        &vx_home,
        &[
            "shim",
            "add",
            "jq",
            "--dir",
            &shim_dir.display().to_string(),
        ],
    )
    .expect("failed to run vx shim add");
    assert_success(&output, "vx shim add jq");

    for file in expected_files(&shim_dir, "jq") {
        assert!(file.exists(), "expected {} to exist", file.display());
        let content = std::fs::read_to_string(&file).expect("failed to read shim");
        assert!(
            content.contains(VX_SHIM_MARKER),
            "missing vx marker in {}:\n{}",
            file.display(),
            content
        );
        assert!(content.contains("jq"), "shim must forward to vx jq");
    }
}

#[test]
fn test_shim_add_registers_the_shim() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let shim_dir = temp.path().join("shims");

    let output = run_vx(
        &vx_home,
        &[
            "shim",
            "add",
            "jq",
            "--dir",
            &shim_dir.display().to_string(),
        ],
    )
    .expect("failed to run vx shim add");
    assert_success(&output, "vx shim add jq");

    let registry = ShimRegistry::load(&vx_home.join("config").join("command-shims.json"))
        .expect("failed to load registry");
    let entry = registry.get("jq").expect("jq should be registered");
    assert_eq!(entry.runtime, "jq");
    assert!(entry.launcher.exists(), "launcher must be the vx binary");
}

#[test]
fn test_shim_list_reports_the_shim() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let shim_dir = temp.path().join("shims");

    run_vx(
        &vx_home,
        &[
            "shim",
            "add",
            "jq",
            "--dir",
            &shim_dir.display().to_string(),
        ],
    )
    .expect("failed to run vx shim add");

    let output = run_vx(&vx_home, &["shim", "list"]).expect("failed to run vx shim list");
    assert_success(&output, "vx shim list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("jq"), "list output was:\n{}", stdout);
}

#[test]
fn test_shim_remove_deletes_only_the_vx_shim() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let shim_dir = temp.path().join("shims");

    run_vx(
        &vx_home,
        &[
            "shim",
            "add",
            "jq",
            "--dir",
            &shim_dir.display().to_string(),
        ],
    )
    .expect("failed to run vx shim add");

    // A hand-written wrapper under a different name must survive the removal
    let user_wrapper = shim_dir.join("user-tool");
    std::fs::write(&user_wrapper, "#!/bin/sh\necho mine\n").expect("failed to write wrapper");

    let output = run_vx(&vx_home, &["shim", "remove", "jq"]).expect("failed to run vx shim remove");
    assert_success(&output, "vx shim remove jq");

    for file in expected_files(&shim_dir, "jq") {
        assert!(
            !file.exists(),
            "{} should have been removed",
            file.display()
        );
    }
    assert!(user_wrapper.exists(), "user wrapper must not be removed");

    let registry = ShimRegistry::load(&vx_home.join("config").join("command-shims.json"))
        .expect("failed to load registry");
    assert!(!registry.contains("jq"), "registry should be empty");
}

#[test]
fn test_shim_add_rejects_shadowing_without_force() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let shim_dir = temp.path().join("shims");
    std::fs::create_dir_all(&shim_dir).expect("failed to create shim dir");

    // An unmanaged binary that will be found first on PATH
    let decoy = shim_dir.join(if cfg!(windows) {
        "decoy-tool.cmd"
    } else {
        "decoy-tool"
    });
    std::fs::write(&decoy, "not a vx shim").expect("failed to write decoy");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&decoy, std::fs::Permissions::from_mode(0o755))
            .expect("failed to make decoy executable");
    }

    let path = std::env::var("PATH").unwrap_or_default();
    let path = format!(
        "{}{}{}",
        shim_dir.display(),
        if cfg!(windows) { ";" } else { ":" },
        path
    );

    let output = Command::new(vx_binary())
        .args([
            "shim",
            "add",
            "decoy-tool",
            "--dir",
            &shim_dir.display().to_string(),
        ])
        .env("VX_HOME", &vx_home)
        .env("PATH", &path)
        .output()
        .expect("failed to run vx shim add");

    assert!(
        !output.status.success(),
        "shadowing should fail without --force"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--force"),
        "expected a --force hint, got:\n{}",
        stderr
    );

    let remaining = std::fs::read_to_string(&decoy).expect("failed to read decoy");
    assert_eq!(remaining, "not a vx shim", "decoy must be untouched");
}

#[test]
fn test_shim_sync_rewrites_existing_shims() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let shim_dir = temp.path().join("shims");

    run_vx(
        &vx_home,
        &[
            "shim",
            "add",
            "jq",
            "--dir",
            &shim_dir.display().to_string(),
        ],
    )
    .expect("failed to run vx shim add");

    // Simulate a shim broken by hand or by a vx upgrade
    for file in expected_files(&shim_dir, "jq") {
        std::fs::write(&file, "stale").expect("failed to overwrite shim");
    }

    let output = run_vx(&vx_home, &["shim", "sync"]).expect("failed to run vx shim sync");
    assert_success(&output, "vx shim sync");

    for file in expected_files(&shim_dir, "jq") {
        let content = std::fs::read_to_string(&file).expect("failed to read shim");
        assert!(
            content.contains(VX_SHIM_MARKER),
            "sync did not rewrite {}",
            file.display()
        );
    }
}

#[test]
fn test_shim_path_reports_the_target_directories() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");

    let output = run_vx(&vx_home, &["shim", "path"]).expect("failed to run vx shim path");
    assert_success(&output, "vx shim path");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("bin"),
        "expected the vx bin dir, got:\n{}",
        stdout
    );
}
