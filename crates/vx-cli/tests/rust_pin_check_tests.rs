//! End-to-end tests for `vx check` on a numeric rust pin with no `rust-toolchain` file.
//!
//! These cover the exact reproduction from the issue: a repository declaring
//! `rust = "1.93.1"` and providing no toolchain file used to get no warning at all,
//! and `vx check` still exited 0.
//!
//! Contract under test:
//!
//! * declared != actual → a visible signal on stderr, and `vx check` exits 0 by
//!   default (warn is the non-breaking default);
//! * with `[check] toolchain_pin_mismatch = "error"` → `vx check` exits non-zero;
//! * declared == actual → no mismatch message at all.

#![allow(clippy::unwrap_used)]

mod common;

use common::{stderr_str, stdout_str};

use std::path::Path;
use std::process::{Command, Output};

/// Serializes tests that must clear `RUSTUP_TOOLCHAIN`.
///
/// `cargo test` runs under rustup's `cargo` shim, which exports `RUSTUP_TOOLCHAIN`.
/// That variable makes rustup own the toolchain by rule 2 of the ownership policy, and
/// a channel name never conflicts with a numeric pin — so the very scenario under test
/// (no toolchain file, numeric pin) is masked. Clearing it reproduces what a plain
/// `vx check` in a terminal sees.
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Run `vx check` in `dir` with `RUSTUP_TOOLCHAIN` removed from the child environment.
fn check_without_toolchain_env(dir: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new(common::vx_binary());
    cmd.args(args)
        .current_dir(dir)
        .env_remove("RUSTUP_TOOLCHAIN")
        .stdin(std::process::Stdio::null());

    cmd.output().expect("vx check should run")
}

/// Write a `vx.toml` declaring the given rust pin, with no `rust-toolchain` file.
fn project_with_pin(dir: &Path, rust_pin: &str, extra: &str) {
    let config = format!("[tools]\nrust = \"{rust_pin}\"\n{extra}");
    std::fs::write(dir.join("vx.toml"), config).unwrap();

    // Belt and braces: the reproduction depends on there being no toolchain file.
    let _ = std::fs::remove_file(dir.join("rust-toolchain.toml"));
    let _ = std::fs::remove_file(dir.join("rust-toolchain"));
}

/// The version the toolchain under test actually reports, resolved the way the
/// code under test resolves it.
///
/// Both halves matter. `cargo test` runs under rustup's shim, so a plain `rustc`
/// inherits the suite's cwd — which sits under a repository root carrying a
/// `rust-toolchain.toml`, so rustup resolves *that* channel. The code under test
/// resolves from `project_root` with `RUSTUP_TOOLCHAIN` removed instead. Ask the
/// question differently and the oracle reports a version the subject never sees,
/// which makes the assertions below fail deterministically.
fn actual_rustc_version(dir: &Path) -> Option<String> {
    let output = std::process::Command::new("rustc")
        .arg("--version")
        .current_dir(dir)
        .env_remove("RUSTUP_TOOLCHAIN")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.split_whitespace().nth(1).map(str::to_string)
}

/// The regression: a pin that does not match produces a visible stderr signal.
#[test]
fn test_mismatched_pin_warns() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(dir.path(), "1.0.0", "");

    let Some(actual) = actual_rustc_version(dir.path()) else {
        eprintln!("skipping: rustc not available");
        return;
    };

    let output = check_without_toolchain_env(dir.path(), &["check"]);
    let stderr = stderr_str(&output);
    let stdout = stdout_str(&output);

    // The signal has to name both the declared and the actual value.
    assert!(
        stderr.contains("1.0.0") || stdout.contains("1.0.0"),
        "declared pin must be visible. stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains(&actual) || stdout.contains(&actual),
        "actual version {actual} must be visible. stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

/// Default severity is warn, so a drifting repository is not broken by upgrading.
#[test]
fn test_mismatched_pin_exits_zero_by_default() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(dir.path(), "1.0.0", "");

    if actual_rustc_version(dir.path()).is_none() {
        eprintln!("skipping: rustc not available");
        return;
    }

    let output = check_without_toolchain_env(dir.path(), &["check"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "warn is the default severity, so a mismatch must not fail the command. stderr:\n{}",
        stderr_str(&output)
    );
}

/// Opting in to `"error"` is what makes the pin enforced.
#[test]
fn test_mismatched_pin_exits_non_zero_when_configured_as_error() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(
        dir.path(),
        "1.0.0",
        "\n[check]\ntoolchain_pin_mismatch = \"error\"\n",
    );

    if actual_rustc_version(dir.path()).is_none() {
        eprintln!("skipping: rustc not available");
        return;
    }

    let output = check_without_toolchain_env(dir.path(), &["check"]);

    assert_ne!(
        output.status.code(),
        Some(0),
        "with toolchain_pin_mismatch = \"error\" a mismatch must fail. stdout:\n{}\nstderr:\n{}",
        stdout_str(&output),
        stderr_str(&output)
    );
}

/// The zero-noise requirement: a correctly pinned repository sees no mismatch message.
#[test]
fn test_matching_pin_is_silent() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();

    let Some(actual) = actual_rustc_version(dir.path()) else {
        eprintln!("skipping: rustc not available");
        return;
    };

    project_with_pin(dir.path(), &actual, "");

    let output = check_without_toolchain_env(dir.path(), &["check"]);
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        !combined.contains("the vx.toml pin is ignored"),
        "a pin equal to the actual version must not warn. output:\n{combined}"
    );
}

/// `rustup-managed` is an ownership declaration, not a version — it must never be
/// reported as a mismatch.
#[test]
fn test_rustup_managed_pin_is_never_a_mismatch() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(dir.path(), "rustup-managed", "");

    let output = check_without_toolchain_env(dir.path(), &["check"]);
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        !combined.contains("the vx.toml pin is ignored"),
        "rustup-managed is not a version and must not mismatch. output:\n{combined}"
    );
}

/// `"ignore"` must actually silence the mismatch — the level is documented as such,
/// and without this case the level can be silently unwired on the CLI side.
#[test]
fn test_mismatched_pin_is_silent_when_configured_as_ignore() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(
        dir.path(),
        "1.0.0",
        "\n[check]\ntoolchain_pin_mismatch = \"ignore\"\n",
    );

    if actual_rustc_version(dir.path()).is_none() {
        eprintln!("skipping: rustc not available");
        return;
    }

    let output = check_without_toolchain_env(dir.path(), &["check"]);
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        !combined.contains("the vx.toml pin is ignored"),
        "toolchain_pin_mismatch = \"ignore\" must suppress the mismatch. output:\n{combined}"
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "ignore must not fail the command"
    );
}

/// `vx check` must report the declared and actual values as two distinct fields.
#[test]
fn test_check_json_reports_declared_and_actual() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(dir.path(), "1.0.0", "");

    let Some(actual) = actual_rustc_version(dir.path()) else {
        eprintln!("skipping: rustc not available");
        return;
    };

    let output = check_without_toolchain_env(dir.path(), &["check", "--json"]);
    let stdout = stdout_str(&output);

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("vx check --json must emit JSON. error: {e}\nstdout:\n{stdout}")
    });

    let requirements = parsed["requirements"]
        .as_array()
        .expect("requirements array")
        .clone();

    let rust = requirements
        .iter()
        .find(|r| r["runtime"] == "rust")
        .unwrap_or_else(|| panic!("rust requirement missing from {requirements:?}"));

    // Two distinct fields: what was declared, and what will actually run.
    assert_eq!(rust["required"], "1.0.0");
    assert_eq!(
        rust["installed"].as_str().unwrap_or_default(),
        actual,
        "installed must report the actual toolchain version"
    );
}
