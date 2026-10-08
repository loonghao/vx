//! Integration coverage for `vx shim` (RFC 0042).
//!
//! These tests drive the real `vx` binary against a temporary `VX_HOME` and
//! write shims into a temporary directory, so they never touch the developer's
//! PATH or the `~/.local/bin` wrapper a user may already have.
//!
//! Shim names are randomized per test. `vx shim add` refuses to shadow a
//! command it did not create, and CI runners ship common runtimes (`jq`,
//! `git`, `node`) on PATH, so a fixed name would fail wherever the runner
//! happens to provide it. See [`test_shim_add_rejects_shadowing_without_force`]
//! for the deliberate coverage of that guard.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{assert_success, init_test_env, vx_available, vx_binary};
use tempfile::TempDir;
use vx_runtime::{ShimRegistry, VX_SHIM_MARKER};

/// The runtime every shim in this file forwards to.
///
/// Only the *shim name* must be collision-free; the runtime is what the shim
/// invokes, so a runtime that is already on PATH is fine.
const RUNTIME: &str = "jq";

fn run_vx(vx_home: &Path, args: &[&str]) -> std::io::Result<Output> {
    Command::new(vx_binary())
        .args(args)
        .env("VX_HOME", vx_home)
        .output()
}

/// A shim name no runner or developer machine is expected to provide.
///
/// Prefixed and suffixed with a UUID so it cannot collide with a real command,
/// which keeps `vx shim add` away from the shadow guard in tests that are not
/// testing the guard itself.
fn unique_name() -> String {
    format!("vx-shim-test-{}", uuid::Uuid::new_v4().simple())
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
    let name = unique_name();

    let output = run_vx(
        &vx_home,
        &[
            "shim",
            "add",
            RUNTIME,
            "--as",
            &name,
            "--dir",
            &shim_dir.display().to_string(),
        ],
    )
    .expect("failed to run vx shim add");
    assert_success(&output, &format!("vx shim add {RUNTIME} --as {name}"));

    for file in expected_files(&shim_dir, &name) {
        assert!(file.exists(), "expected {} to exist", file.display());
        let content = std::fs::read_to_string(&file).expect("failed to read shim");
        assert!(
            content.contains(VX_SHIM_MARKER),
            "missing vx marker in {}:\n{}",
            file.display(),
            content
        );
        assert!(
            content.contains(RUNTIME),
            "shim must forward to vx {RUNTIME}"
        );
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
    let name = unique_name();

    let output = run_vx(
        &vx_home,
        &[
            "shim",
            "add",
            RUNTIME,
            "--as",
            &name,
            "--dir",
            &shim_dir.display().to_string(),
        ],
    )
    .expect("failed to run vx shim add");
    assert_success(&output, &format!("vx shim add {RUNTIME} --as {name}"));

    let registry = ShimRegistry::load(&vx_home.join("config").join("command-shims.json"))
        .expect("failed to load registry");
    let entry = registry
        .get(&name)
        .unwrap_or_else(|| panic!("{name} should be registered"));
    assert_eq!(entry.runtime, RUNTIME);
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
    let name = unique_name();

    run_vx(
        &vx_home,
        &[
            "shim",
            "add",
            RUNTIME,
            "--as",
            &name,
            "--dir",
            &shim_dir.display().to_string(),
        ],
    )
    .expect("failed to run vx shim add");

    let output = run_vx(&vx_home, &["shim", "list"]).expect("failed to run vx shim list");
    assert_success(&output, "vx shim list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&name), "list output was:\n{}", stdout);
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
    let name = unique_name();

    run_vx(
        &vx_home,
        &[
            "shim",
            "add",
            RUNTIME,
            "--as",
            &name,
            "--dir",
            &shim_dir.display().to_string(),
        ],
    )
    .expect("failed to run vx shim add");

    // A hand-written wrapper under a different name must survive the removal
    let user_wrapper = shim_dir.join("user-tool");
    std::fs::write(&user_wrapper, "#!/bin/sh\necho mine\n").expect("failed to write wrapper");

    let output =
        run_vx(&vx_home, &["shim", "remove", &name]).expect("failed to run vx shim remove");
    assert_success(&output, &format!("vx shim remove {name}"));

    for file in expected_files(&shim_dir, &name) {
        assert!(
            !file.exists(),
            "{} should have been removed",
            file.display()
        );
    }
    assert!(user_wrapper.exists(), "user wrapper must not be removed");

    let registry = ShimRegistry::load(&vx_home.join("config").join("command-shims.json"))
        .expect("failed to load registry");
    assert!(!registry.contains(&name), "registry should be empty");
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
            RUNTIME,
            "--as",
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
fn test_shim_add_force_shadows_without_overwriting_an_existing_command() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let shim_dir = temp.path().join("shims");
    std::fs::create_dir_all(&shim_dir).expect("failed to create shim dir");
    let existing_dir = temp.path().join("existing-bin");
    std::fs::create_dir_all(&existing_dir).expect("failed to create existing command dir");

    let name = unique_name();
    let decoy = existing_dir.join(if cfg!(windows) {
        format!("{name}.cmd")
    } else {
        name.clone()
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
        existing_dir.display(),
        if cfg!(windows) { ";" } else { ":" },
        path
    );

    let refused = Command::new(vx_binary())
        .args([
            "shim",
            "add",
            RUNTIME,
            "--as",
            &name,
            "--force",
            "--dir",
            &existing_dir.display().to_string(),
        ])
        .env("VX_HOME", &vx_home)
        .env("PATH", &path)
        .output()
        .expect("failed to run vx shim add against an unmanaged destination");
    assert!(
        !refused.status.success(),
        "--force must not overwrite an unmanaged destination"
    );
    assert_eq!(std::fs::read_to_string(&decoy).unwrap(), "not a vx shim");
    assert!(
        ShimRegistry::load(&vx_home.join("config").join("command-shims.json"))
            .unwrap()
            .is_empty()
    );

    let output = Command::new(vx_binary())
        .args([
            "shim",
            "add",
            RUNTIME,
            "--as",
            &name,
            "--force",
            "--dir",
            &shim_dir.display().to_string(),
        ])
        .env("VX_HOME", &vx_home)
        .env("PATH", &path)
        .output()
        .expect("failed to run vx shim add");

    assert_success(
        &output,
        &format!("vx shim add {RUNTIME} --as {name} --force"),
    );

    for file in expected_files(&shim_dir, &name) {
        let content = std::fs::read_to_string(&file).expect("failed to read shim");
        assert!(
            content.contains(VX_SHIM_MARKER),
            "--force did not create a separate command shim {}:\n{}",
            file.display(),
            content
        );
    }
    assert_eq!(
        std::fs::read_to_string(&decoy).unwrap(),
        "not a vx shim",
        "PATH shadowing must preserve the original command"
    );
}

#[test]
fn test_shim_sync_restores_missing_shims_and_preserves_user_edits() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let shim_dir = temp.path().join("shims");
    let name = unique_name();

    let added = run_vx(
        &vx_home,
        &[
            "shim",
            "add",
            RUNTIME,
            "--as",
            &name,
            "--dir",
            &shim_dir.display().to_string(),
        ],
    )
    .expect("failed to run vx shim add");
    assert_success(&added, "create registered shims before syncing");

    let files: Vec<_> = expected_files(&shim_dir, &name)
        .into_iter()
        .map(|file| {
            let contents = std::fs::read_to_string(&file).expect("failed to read original shim");
            (file, contents)
        })
        .collect();
    // A missing registered variant can be restored while existing owned files
    // continue to match the record; user edits are a different ownership state.
    std::fs::remove_file(&files[0].0).expect("failed to remove a registered shim variant");

    let output = run_vx(&vx_home, &["shim", "sync"]).expect("failed to run vx shim sync");
    assert_success(&output, "vx shim sync");

    for (file, expected) in &files {
        assert_eq!(
            std::fs::read_to_string(file).unwrap(),
            *expected,
            "sync did not restore {}",
            file.display(),
        );
    }
    let edited = format!("{}\n# user customization\n", files[0].1);
    std::fs::write(&files[0].0, &edited).unwrap();
    let refused = run_vx(&vx_home, &["shim", "sync"]).expect("failed to run vx shim sync");
    assert!(
        !refused.status.success(),
        "sync must refuse to overwrite an edited registered wrapper"
    );
    assert_eq!(std::fs::read_to_string(&files[0].0).unwrap(), edited);
    for (file, expected) in files.iter().skip(1) {
        assert_eq!(std::fs::read_to_string(file).unwrap(), *expected);
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
