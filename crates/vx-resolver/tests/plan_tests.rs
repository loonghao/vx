//! Tests for ExecutionPlan and PlannedRuntime
//!
//! Includes regression tests for the version propagation bug where
//! `mark_installed_with_version` was not called after installation,
//! causing re-resolution to search for a non-existent "latest" directory
//! in the store (e.g., `~/.vx/store/uv/latest/` instead of `~/.vx/store/uv/0.10.0/`).

use rstest::rstest;
use std::path::PathBuf;
use vx_resolver::{
    ExecutionConfig, ExecutionPlan, InstallStatus, PlannedRuntime, VersionResolution, VersionSource,
};

// ============================================================
// ExecutionPlan tests
// ============================================================

#[test]
fn test_execution_plan_new() {
    let primary = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/usr/local/bin/node"),
    );
    let config = ExecutionConfig::with_args(vec!["--version".to_string()]);

    let plan = ExecutionPlan::new(primary, config);

    assert_eq!(plan.primary.name, "node");
    assert!(plan.dependencies.is_empty());
    assert!(plan.injected.is_empty());
    assert!(plan.proxy.is_none());
    assert!(!plan.needs_install());
}

#[test]
fn test_execution_plan_with_dependencies() {
    let primary = PlannedRuntime::needs_install("npm", "10.0.0".to_string());
    let node_dep = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/usr/local/bin/node"),
    );

    let plan = ExecutionPlan::new(primary, ExecutionConfig::default()).with_dependency(node_dep);

    assert_eq!(plan.dependencies.len(), 1);
    assert_eq!(plan.dependencies[0].name, "node");
    assert!(plan.needs_install());
    assert_eq!(plan.runtimes_needing_install().len(), 1);
    assert_eq!(plan.runtimes_needing_install()[0].name, "npm");
}

#[test]
fn test_execution_plan_unsupported_runtimes() {
    let primary = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/usr/local/bin/node"),
    );
    let unsupported = PlannedRuntime::unsupported("msvc", "Windows only".to_string());

    let plan = ExecutionPlan::new(primary, ExecutionConfig::default()).with_injected(unsupported);

    assert_eq!(plan.unsupported_runtimes().len(), 1);
    assert_eq!(plan.unsupported_runtimes()[0].name, "msvc");
}

#[test]
fn test_all_runtimes_iterator() {
    let primary = PlannedRuntime::installed(
        "npm",
        "10.0.0".to_string(),
        PathBuf::from("/usr/local/bin/npm"),
    );
    let dep = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/usr/local/bin/node"),
    );
    let injected = PlannedRuntime::installed(
        "yarn",
        "4.0.0".to_string(),
        PathBuf::from("/usr/local/bin/yarn"),
    );

    let plan = ExecutionPlan::new(primary, ExecutionConfig::default())
        .with_dependency(dep)
        .with_injected(injected);

    let names: Vec<&str> = plan.all_runtimes().map(|r| r.name.as_str()).collect();
    // Order: deps → primary → injected
    assert_eq!(names, vec!["node", "npm", "yarn"]);
}

// ============================================================
// PlannedRuntime tests
// ============================================================

#[test]
fn test_planned_runtime_installed() {
    let rt = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/usr/local/bin/node"),
    );

    assert_eq!(rt.name, "node");
    assert_eq!(rt.version_string(), Some("20.0.0"));
    assert!(rt.is_ready());
    assert_eq!(rt.status, InstallStatus::Installed);
}

#[test]
fn test_planned_runtime_needs_install() {
    let rt = PlannedRuntime::needs_install("go", "1.21.0".to_string());

    assert_eq!(rt.name, "go");
    assert_eq!(rt.version_string(), Some("1.21.0"));
    assert!(!rt.is_ready());
    assert_eq!(rt.status, InstallStatus::NeedsInstall);
}

#[test]
fn test_planned_runtime_unsupported() {
    let rt = PlannedRuntime::unsupported("msvc", "Windows only".to_string());

    assert_eq!(rt.name, "msvc");
    assert!(rt.version_string().is_none());
    assert!(!rt.is_ready());
    assert!(matches!(
        rt.status,
        InstallStatus::PlatformUnsupported { .. }
    ));
}

#[test]
fn test_planned_runtime_mark_installed() {
    let mut rt = PlannedRuntime::needs_install("node", "20.0.0".to_string());
    assert!(!rt.is_ready());

    rt.mark_installed(PathBuf::from("/home/user/.vx/store/node/20.0.0/bin/node"));
    assert!(rt.is_ready());
    assert_eq!(rt.status, InstallStatus::Installed);
}

// ============================================================
// VersionResolution tests
// ============================================================

#[test]
fn test_version_resolution_variants() {
    let installed = VersionResolution::Installed {
        version: "20.0.0".to_string(),
        source: VersionSource::Explicit,
    };
    assert_ne!(installed, VersionResolution::Unresolved);

    let range = VersionResolution::Range {
        spec: "^20.0.0".to_string(),
        resolved: "20.5.1".to_string(),
    };
    assert_ne!(range, installed);
}

// ============================================================
// Regression tests: mark_installed_with_version
//
// These tests verify that after calling `mark_installed_with_version`,
// the version is updated to the concrete installed version.
// This prevents the bug where re-resolution after installation would
// search for `~/.vx/store/uv/latest/` instead of `~/.vx/store/uv/0.10.0/`.
// ============================================================

#[rstest]
#[case::latest("latest", "0.10.0")]
#[case::specific("20.0.0", "20.0.0")]
#[case::prefix("20", "20.18.1")]
#[case::range("^1.0.0", "1.5.3")]
fn test_mark_installed_with_version_updates_version(
    #[case] requested_version: &str,
    #[case] actual_version: &str,
) {
    let mut rt = PlannedRuntime::needs_install("uv", requested_version.to_string());

    // Before: status is NeedsInstall, version is the requested version
    assert_eq!(rt.status, InstallStatus::NeedsInstall);
    assert_eq!(rt.version_string(), Some(requested_version));

    // Simulate what EnsureStage does after installation
    rt.mark_installed_with_version(actual_version.to_string(), None);

    // After: status is Installed, version is the ACTUAL installed version
    assert_eq!(rt.status, InstallStatus::Installed);
    assert_eq!(rt.version_string(), Some(actual_version));
    assert_eq!(
        rt.version,
        VersionResolution::Installed {
            version: actual_version.to_string(),
            source: VersionSource::VxManaged,
        }
    );
}

#[test]
fn test_mark_installed_with_version_sets_executable() {
    let mut rt = PlannedRuntime::needs_install("node", "latest".to_string());
    assert!(rt.executable.is_none());

    rt.mark_installed_with_version(
        "20.18.0".to_string(),
        Some(PathBuf::from("/home/user/.vx/store/node/20.18.0/bin/node")),
    );

    assert!(rt.is_ready());
    assert_eq!(
        rt.executable,
        Some(PathBuf::from(
            "/home/user/.vx/store/node/20.18.0/bin/node"
        ))
    );
    assert_eq!(rt.version_string(), Some("20.18.0"));
}

#[test]
fn test_mark_installed_with_version_without_executable() {
    let mut rt = PlannedRuntime::needs_install("uv", "latest".to_string());

    // No executable provided (will be resolved by re-resolution later)
    rt.mark_installed_with_version("0.10.0".to_string(), None);

    assert_eq!(rt.status, InstallStatus::Installed);
    assert_eq!(rt.version_string(), Some("0.10.0"));
    // executable is still None — not "ready" yet, but version is correct for re-resolution
    assert!(rt.executable.is_none());
    assert!(!rt.is_ready());
}

/// Regression test: Verify that after mark_installed_with_version,
/// `version_string()` returns the concrete version, not "latest".
/// This is the exact scenario that caused the CI failure:
/// `resolve_with_version("uv", Some("latest"))` searched for a
/// non-existent directory `~/.vx/store/uv/latest/windows-x64/`.
#[test]
fn test_regression_version_string_after_install_not_latest() {
    let mut rt = PlannedRuntime::needs_install("uv", "latest".to_string());
    assert_eq!(rt.version_string(), Some("latest"));

    // EnsureStage calls mark_installed_with_version after successful install
    rt.mark_installed_with_version("0.10.0".to_string(), None);

    // version_string() must now return "0.10.0", NOT "latest"
    assert_ne!(rt.version_string(), Some("latest"));
    assert_eq!(rt.version_string(), Some("0.10.0"));
}

/// Regression test: Verify version propagation works for dependency runtimes too.
/// When `npm` (dependency) is installed, its version should be updated from
/// "latest" to the concrete version.
#[test]
fn test_regression_dependency_version_updated_after_install() {
    let primary = PlannedRuntime::needs_install("npm", "latest".to_string());
    let mut dep = PlannedRuntime::needs_install("node", "latest".to_string());

    let mut plan = ExecutionPlan::new(primary, ExecutionConfig::default()).with_dependency(dep.clone());

    // Simulate EnsureStage: install dependency first
    dep.mark_installed_with_version("20.18.0".to_string(), None);
    plan.dependencies[0] = dep;

    // Dependency version should be concrete
    assert_eq!(plan.dependencies[0].version_string(), Some("20.18.0"));
    assert_eq!(plan.dependencies[0].status, InstallStatus::Installed);

    // Primary is still NeedsInstall
    assert_eq!(plan.primary.version_string(), Some("latest"));
    assert_eq!(plan.primary.status, InstallStatus::NeedsInstall);

    // Simulate EnsureStage: install primary
    plan.primary
        .mark_installed_with_version("10.9.0".to_string(), None);
    assert_eq!(plan.primary.version_string(), Some("10.9.0"));
    assert_eq!(plan.primary.status, InstallStatus::Installed);
}

/// Regression test: Verify version propagation for injected (--with) runtimes.
#[test]
fn test_regression_injected_version_updated_after_install() {
    let primary = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/usr/local/bin/node"),
    );
    let mut injected = PlannedRuntime::needs_install("yarn", "latest".to_string());

    let mut plan =
        ExecutionPlan::new(primary, ExecutionConfig::default()).with_injected(injected.clone());

    // Simulate EnsureStage: install injected runtime
    injected.mark_installed_with_version("4.5.0".to_string(), None);
    plan.injected[0] = injected;

    assert_eq!(plan.injected[0].version_string(), Some("4.5.0"));
    assert_eq!(plan.injected[0].status, InstallStatus::Installed);
    assert!(!plan.needs_install());
}
