//! Regression tests: a numeric `vx.toml` rust pin must be resolvable without a
//! `rust-toolchain` file.
//!
//! This is the second half of the ownership work. The first half made vx defer to
//! rustup whenever something *names* a toolchain — a `rust-toolchain.toml`, an exported
//! `RUSTUP_TOOLCHAIN`, or `rust = "rustup-managed"`. That left a gap: with none of those
//! present, `RustToolchainOwner::channel()` returns `None`, so there was no version to
//! compare against and a declared `rust = "1.93.1"` was silently ignored while the build
//! ran on stable. No warning, and `vx check` still exited 0.
//!
//! These tests pin the contract for the gap:
//!
//! 1. With no toolchain file anywhere, the effective toolchain is still resolvable.
//! 2. A declared pin that differs from it is a detectable conflict.
//! 3. A declared pin that matches it is *not* a conflict — correctly pinned
//!    repositories must never be noisy.
//!
//! The comparison under test is [`vx_resolver::versions_conflict`], driven by
//! [`vx_resolver::effective_toolchain_version`]; together they are what makes the pin
//! meaningful in the no-toolchain-file case.

use std::collections::HashMap;

use vx_resolver::{
    RUST_TOOLCHAIN_LEGACY, RUST_TOOLCHAIN_TOML, RUSTUP_MANAGED, detect_toolchain_owner_with_env,
    effective_toolchain_version, parse_rustc_version, versions_conflict,
};

/// Serializes access to the process environment, which is process-global.
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The regression: with no toolchain file and no `RUSTUP_TOOLCHAIN`, ownership falls to
/// vx and carries no channel — so the pin must come from the resolved toolchain.
#[test]
fn test_no_toolchain_file_yields_vx_owner_without_channel() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();

    let owner = detect_toolchain_owner_with_env(dir.path(), &HashMap::new());

    assert!(!owner.is_rustup_managed());
    assert_eq!(
        owner.channel(),
        None,
        "the gap: no override means no named channel to validate a pin against"
    );
}

/// A toolchain file anywhere up the tree still wins over the fallback.
#[test]
fn test_toolchain_file_still_takes_precedence() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(RUST_TOOLCHAIN_TOML),
        "[toolchain]\nchannel = \"1.93.1\"\n",
    )
    .unwrap();

    let owner = detect_toolchain_owner_with_env(dir.path(), &HashMap::new());

    assert_eq!(owner.channel(), Some("1.93.1"));
}

/// Legacy `rust-toolchain` files are honoured too, so the fallback is not reached.
#[test]
fn test_legacy_toolchain_file_still_takes_precedence() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(RUST_TOOLCHAIN_LEGACY), "1.93.1\n").unwrap();

    let owner = detect_toolchain_owner_with_env(dir.path(), &HashMap::new());

    assert_eq!(owner.channel(), Some("1.93.1"));
}

/// The core fix: even with no toolchain file, a real version is resolvable, so a
/// declared pin finally has something to be compared against.
#[test]
fn test_effective_version_resolvable_without_toolchain_file() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();

    if !which::which("rustc").is_ok() {
        eprintln!("skipping: rustc not on PATH");
        return;
    }

    let effective = effective_toolchain_version(dir.path())
        .expect("a toolchain is installed, so an effective version must resolve");

    assert!(!effective.is_empty());
    assert!(
        effective.starts_with(|c: char| c.is_ascii_digit()),
        "expected a numeric version, got {effective:?}"
    );
}

/// A declared pin that disagrees with the resolved toolchain is a conflict — this is
/// the signal that used to be missing entirely.
#[test]
fn test_declared_pin_conflicts_with_effective_toolchain() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();

    let Some(effective) = effective_toolchain_version(dir.path()) else {
        eprintln!("skipping: no rust toolchain resolved");
        return;
    };

    // "1.0.0" predates any supported toolchain, so it must conflict with whatever runs.
    // This is the positive half of the contract: without it, a mutation that makes
    // conflict detection always return false would still pass the suite.
    assert!(
        versions_conflict("1.0.0", &effective),
        "a 1.0.0 pin must conflict with the effective toolchain {effective}"
    );

    // And the same comparison must not fire for an equal pin, so the two assertions
    // together pin both directions.
    assert!(
        !versions_conflict(&effective, &effective),
        "an equal pin must not conflict with {effective}"
    );
}

/// The zero-noise requirement: a pin equal to the resolved toolchain is not a conflict,
/// so already-correct repositories are never spammed.
#[test]
fn test_declared_pin_matching_effective_toolchain_is_silent() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();

    let Some(effective) = effective_toolchain_version(dir.path()) else {
        eprintln!("skipping: no rust toolchain resolved");
        return;
    };

    assert!(
        !versions_conflict(&effective, &effective),
        "a pin equal to the effective toolchain must not conflict"
    );
}

/// The `rustup-managed` sentinel is an ownership declaration, not a version: it must
/// never be reported as a conflict.
#[test]
fn test_rustup_managed_sentinel_never_conflicts() {
    assert!(!versions_conflict(RUSTUP_MANAGED, "1.98.0"));
    assert!(!versions_conflict("1.98.0", RUSTUP_MANAGED));
}

/// Channels name no concrete version, so warning about them would be pure noise.
#[test]
fn test_channels_never_conflict() {
    for channel in ["stable", "beta", "nightly", "latest", "*"] {
        assert!(
            !versions_conflict(channel, "1.98.0"),
            "{channel} must not conflict"
        );
        assert!(
            !versions_conflict("1.98.0", channel),
            "{channel} must not conflict"
        );
    }
}

/// `rustc --version` output carries a trailing build hash that must be dropped.
#[rstest::rstest]
#[case("rustc 1.98.0 (abc123 2026-01-01)\n", Some("1.98.0"))]
#[case("rustc 1.93.1\n", Some("1.93.1"))]
#[case("rustc 1.98.0-beta.1 (x 2026-01-01)\n", Some("1.98.0-beta.1"))]
#[case("", None)]
#[case("rustc\n", None)]
#[case("rustup 1.28.1\n", Some("1.28.1"))]
fn test_parse_rustc_version(#[case] stdout: &str, #[case] expected: Option<&str>) {
    assert_eq!(parse_rustc_version(stdout).as_deref(), expected);
}

/// Missing patch components are zero, so `1.83` and `1.83.0` agree — otherwise a
/// two-component pin would look permanently broken.
#[test]
fn test_partial_pins_do_not_false_positive() {
    assert!(!versions_conflict("1.83", "1.83.0"));
    assert!(!versions_conflict("1.83.0", "1.83"));
    assert!(versions_conflict("1.83", "1.90.0"));
}
