//! Tests for InstallResult constructors
//!
//! Validates all InstallResult construction methods, including the new
//! `proxy()` and `already_installed_with()` methods added to fix the
//! "executable_path discarded after installation" design flaw.

use std::path::PathBuf;
use vx_runtime::InstallResult;

/// Helper to create a cross-platform absolute path for testing.
fn abs_path(segments: &[&str]) -> PathBuf {
    if cfg!(windows) {
        let mut p = PathBuf::from("C:\\");
        for s in segments {
            p.push(s);
        }
        p
    } else {
        let mut p = PathBuf::from("/");
        for s in segments {
            p.push(s);
        }
        p
    }
}

// ============================================================
// InstallResult::success
// ============================================================

#[test]
fn test_install_result_success() {
    let install_path = abs_path(&[".vx", "store", "uv", "0.6.12"]);
    let exe_path = abs_path(&[".vx", "store", "uv", "0.6.12", "uv"]);
    let result =
        InstallResult::success(install_path.clone(), exe_path.clone(), "0.6.12".to_string());

    assert!(result.success);
    assert!(!result.already_installed);
    assert_eq!(result.version, "0.6.12");
    assert_eq!(result.install_path, install_path);
    assert_eq!(result.executable_path, exe_path);
}

#[test]
fn test_install_result_success_executable_is_absolute() {
    let result = InstallResult::success(
        abs_path(&[".vx", "store", "node", "20.0.0"]),
        abs_path(&[".vx", "store", "node", "20.0.0", "bin", "node"]),
        "20.0.0".to_string(),
    );

    assert!(result.executable_path.is_absolute());
}

// ============================================================
// InstallResult::already_installed
// ============================================================

#[test]
fn test_install_result_already_installed() {
    let result = InstallResult::already_installed(
        abs_path(&[".vx", "store", "go", "1.21.0"]),
        abs_path(&[".vx", "store", "go", "1.21.0", "bin", "go"]),
        "1.21.0".to_string(),
    );

    assert!(result.success);
    assert!(result.already_installed);
    assert_eq!(result.version, "1.21.0");
    assert!(result.executable_path.is_absolute());
}

// ============================================================
// InstallResult::system_installed
// ============================================================

#[test]
fn test_install_result_system_installed_with_path() {
    let exe = abs_path(&["usr", "bin", "jq"]);
    let result = InstallResult::system_installed("1.0.0".to_string(), Some(exe.clone()));

    assert!(result.success);
    assert_eq!(result.version, "1.0.0");
    assert_eq!(result.executable_path, exe);
    assert_eq!(result.install_path, PathBuf::from("system"));
}

#[test]
fn test_install_result_system_installed_without_path() {
    let result = InstallResult::system_installed("2.0.0".to_string(), None);

    assert!(result.success);
    assert_eq!(result.version, "2.0.0");
    assert_eq!(result.executable_path, PathBuf::from("system"));
}

// ============================================================
// InstallResult::proxy (new - RFC 0028)
// ============================================================

#[test]
fn test_install_result_proxy() {
    let result = InstallResult::proxy("4.5.0".to_string());

    assert!(result.success);
    assert!(result.already_installed);
    assert_eq!(result.version, "4.5.0");
    // Proxy runtimes have no executable_path — prepare stage handles execution
    assert!(!result.executable_path.is_absolute());
    assert_eq!(result.executable_path, PathBuf::new());
}

#[test]
fn test_install_result_proxy_executable_not_absolute() {
    // Critical: proxy result's executable_path must NOT be absolute,
    // so EnsureStage correctly skips setting it on PlannedRuntime.
    let result = InstallResult::proxy("1.0.0".to_string());
    assert!(!result.executable_path.is_absolute());
}

// ============================================================
// InstallResult::already_installed_with (new)
// ============================================================

#[test]
fn test_install_result_already_installed_with_executable() {
    let exe = abs_path(&[".vx", "store", "uv", "0.6.12", "uv"]);
    let result = InstallResult::already_installed_with("0.6.12".to_string(), Some(exe.clone()));

    assert!(result.success);
    assert!(result.already_installed);
    assert_eq!(result.version, "0.6.12");
    assert_eq!(result.executable_path, exe);
    assert!(result.executable_path.is_absolute());
}

#[test]
fn test_install_result_already_installed_with_none() {
    // When we can't find the executable, fallback to empty path
    let result = InstallResult::already_installed_with("0.6.12".to_string(), None);

    assert!(result.success);
    assert!(result.already_installed);
    assert_eq!(result.version, "0.6.12");
    assert_eq!(result.executable_path, PathBuf::new());
    // Importantly, this empty path is NOT absolute
    assert!(!result.executable_path.is_absolute());
}

// ============================================================
// Regression: is_absolute check in EnsureStage
//
// EnsureStage uses `result.executable_path.is_absolute()` to decide
// whether to use the path. These tests verify the contract for each
// constructor so that EnsureStage logic is correct for all providers.
// ============================================================

#[test]
fn test_executable_path_is_absolute_for_fresh_install() {
    let result = InstallResult::success(
        abs_path(&[".vx", "store", "uv", "0.6.12"]),
        abs_path(&[".vx", "store", "uv", "0.6.12", "uv.exe"]),
        "0.6.12".to_string(),
    );
    assert!(
        result.executable_path.is_absolute(),
        "Fresh install must have absolute executable_path"
    );
}

#[test]
fn test_executable_path_is_absolute_for_already_installed() {
    let result = InstallResult::already_installed(
        abs_path(&[".vx", "store", "node", "20.0.0"]),
        abs_path(&[".vx", "store", "node", "20.0.0", "node.exe"]),
        "20.0.0".to_string(),
    );
    assert!(
        result.executable_path.is_absolute(),
        "Already-installed must have absolute executable_path"
    );
}

#[test]
fn test_executable_path_not_absolute_for_proxy() {
    let result = InstallResult::proxy("1.0.0".to_string());
    assert!(
        !result.executable_path.is_absolute(),
        "Proxy result must NOT have absolute executable_path"
    );
}

#[test]
fn test_executable_path_not_absolute_for_already_installed_with_none() {
    let result = InstallResult::already_installed_with("1.0.0".to_string(), None);
    assert!(
        !result.executable_path.is_absolute(),
        "already_installed_with(None) must NOT have absolute executable_path"
    );
}
