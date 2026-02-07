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

// ============================================================
// Executable path propagation tests
//
// These tests verify the fix for the "executable_path discarded after
// installation" design flaw. Previously, InstallResult.executable_path
// was thrown away, forcing a re-resolve via filesystem scan which could
// fail (especially on Windows CI). Now, the executable path is passed
// directly from InstallResult to PlannedRuntime.
// ============================================================

/// Test that mark_installed_with_version with an absolute executable
/// path makes the runtime "ready" (is_ready() == true).
#[rstest]
#[case::uv("/home/user/.vx/store/uv/0.6.12/uv", "uv", "0.6.12")]
#[case::node("/home/user/.vx/store/node/20.0.0/bin/node", "node", "20.0.0")]
#[case::go("/home/user/.vx/store/go/1.21.0/bin/go", "go", "1.21.0")]
#[case::npm("/home/user/.vx/store/node/20.0.0/bin/npm", "npm", "10.0.0")]
#[case::bun("/home/user/.vx/store/bun/1.0.0/bun", "bun", "1.0.0")]
fn test_mark_installed_with_executable_makes_ready(
    #[case] exe_path: &str,
    #[case] runtime_name: &str,
    #[case] version: &str,
) {
    let mut rt = PlannedRuntime::needs_install(runtime_name, "latest".to_string());
    assert!(!rt.is_ready());
    assert!(rt.executable.is_none());

    rt.mark_installed_with_version(
        version.to_string(),
        Some(PathBuf::from(exe_path)),
    );

    assert!(rt.is_ready(), "{} should be ready after install", runtime_name);
    assert_eq!(rt.executable, Some(PathBuf::from(exe_path)));
    assert_eq!(rt.version_string(), Some(version));
    assert_eq!(rt.status, InstallStatus::Installed);
}

/// Simulate the full EnsureStage flow for a primary + dependency scenario.
/// This is the exact pattern that was broken: npm (primary) depends on node (dep),
/// both need installation. After fix, both should have executable paths.
#[test]
fn test_full_ensure_flow_primary_with_dependency_executable_propagation() {
    let primary = PlannedRuntime::needs_install("npm", "latest".to_string());
    let dep = PlannedRuntime::needs_install("node", "latest".to_string());

    let mut plan =
        ExecutionPlan::new(primary, ExecutionConfig::default()).with_dependency(dep);

    // Step 1: Install dependency (node) — EnsureStage gets InstallResult with executable_path
    let node_exe = PathBuf::from("/home/user/.vx/store/node/20.18.0/bin/node");
    plan.dependencies[0].mark_installed_with_version(
        "20.18.0".to_string(),
        Some(node_exe.clone()),
    );

    assert!(plan.dependencies[0].is_ready());
    assert_eq!(plan.dependencies[0].executable, Some(node_exe));

    // Step 2: Install primary (npm) — EnsureStage gets InstallResult with executable_path
    let npm_exe = PathBuf::from("/home/user/.vx/store/node/20.18.0/bin/npm");
    plan.primary.mark_installed_with_version(
        "10.9.0".to_string(),
        Some(npm_exe.clone()),
    );

    assert!(plan.primary.is_ready());
    assert_eq!(plan.primary.executable, Some(npm_exe));

    // Both should be ready — no re-resolve needed
    assert!(!plan.needs_install());
}

/// Simulate EnsureStage flow for injected runtimes with executable propagation.
#[test]
fn test_full_ensure_flow_injected_executable_propagation() {
    let primary = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/home/user/.vx/store/node/20.0.0/bin/node"),
    );
    let injected = PlannedRuntime::needs_install("yarn", "latest".to_string());

    let mut plan =
        ExecutionPlan::new(primary, ExecutionConfig::default()).with_injected(injected);

    // Install injected runtime — with executable path
    let yarn_exe = PathBuf::from("/home/user/.vx/store/yarn/4.5.0/bin/yarn");
    plan.injected[0].mark_installed_with_version(
        "4.5.0".to_string(),
        Some(yarn_exe.clone()),
    );

    assert!(plan.injected[0].is_ready());
    assert_eq!(plan.injected[0].executable, Some(yarn_exe));
    assert!(!plan.needs_install());
}

/// Simulate the exact CI failure scenario: uv needs install, after installation
/// the executable path must be propagated to make it ready.
#[test]
fn test_regression_uv_executable_propagated_after_install() {
    let mut rt = PlannedRuntime::needs_install("uv", "latest".to_string());

    // Before: not ready, no executable
    assert!(!rt.is_ready());
    assert!(rt.executable.is_none());

    // Simulate what EnsureStage now does: pass InstallResult.executable_path
    let exe = PathBuf::from("/home/user/.vx/store/uv/0.6.12/uv");
    rt.mark_installed_with_version("0.6.12".to_string(), Some(exe.clone()));

    // After: ready with executable
    assert!(rt.is_ready());
    assert_eq!(rt.executable, Some(exe));
    assert_eq!(rt.version_string(), Some("0.6.12"));
}

/// Simulate the CI failure: uv install returns a platform-specific absolute path
#[test]
fn test_regression_uv_executable_propagated_absolute_path() {
    let mut rt = PlannedRuntime::needs_install("uv", "latest".to_string());

    let exe = abs_path(&[".vx", "store", "uv", "0.6.12", "uv"]);
    rt.mark_installed_with_version("0.6.12".to_string(), Some(exe.clone()));

    assert!(rt.is_ready());
    assert!(rt.executable.as_ref().unwrap().is_absolute());
    assert_eq!(rt.executable, Some(exe));
}

/// When InstallResult has a non-absolute executable_path (e.g., proxy or fallback),
/// EnsureStage should NOT set it — executable stays None.
#[test]
fn test_non_absolute_executable_not_set() {
    let mut rt = PlannedRuntime::needs_install("vite", "latest".to_string());

    // Proxy result has empty executable_path (not absolute)
    rt.mark_installed_with_version("5.0.0".to_string(), None);

    // executable stays None — not "ready" but version is updated
    assert!(rt.executable.is_none());
    assert!(!rt.is_ready());
    assert_eq!(rt.status, InstallStatus::Installed);
    assert_eq!(rt.version_string(), Some("5.0.0"));
}

/// Test that mark_installed_with_version does NOT overwrite an existing
/// executable when called with None.
#[test]
fn test_mark_installed_preserves_existing_executable_when_none() {
    let mut rt = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/usr/local/bin/node"),
    );

    // Re-mark with None should NOT clear the existing executable
    rt.mark_installed_with_version("20.0.0".to_string(), None);

    assert_eq!(
        rt.executable,
        Some(PathBuf::from("/usr/local/bin/node")),
        "Existing executable should be preserved when new value is None"
    );
}

/// Simulate complete multi-runtime installation flow covering all categories:
/// dependencies, primary, and injected — all with executable propagation.
#[test]
fn test_complete_multi_runtime_install_flow() {
    // Scenario: `vx npx create-react-app my-app --with yarn`
    // - dependency: node (needs install)
    // - primary: npx (needs install)
    // - injected: yarn (needs install)
    let primary = PlannedRuntime::needs_install("npx", "latest".to_string());
    let dep = PlannedRuntime::needs_install("node", "latest".to_string());
    let injected = PlannedRuntime::needs_install("yarn", "latest".to_string());

    let mut plan = ExecutionPlan::new(primary, ExecutionConfig::default())
        .with_dependency(dep)
        .with_injected(injected);

    // All need installation
    assert_eq!(plan.runtimes_needing_install().len(), 3);

    // Install dependency (node)
    let node_exe = PathBuf::from("/home/user/.vx/store/node/20.18.0/bin/node");
    plan.dependencies[0].mark_installed_with_version(
        "20.18.0".to_string(),
        Some(node_exe.clone()),
    );
    assert!(plan.dependencies[0].is_ready());

    // Install primary (npx)
    let npx_exe = PathBuf::from("/home/user/.vx/store/node/20.18.0/bin/npx");
    plan.primary.mark_installed_with_version(
        "10.9.0".to_string(),
        Some(npx_exe.clone()),
    );
    assert!(plan.primary.is_ready());

    // Install injected (yarn)
    let yarn_exe = PathBuf::from("/home/user/.vx/store/yarn/4.5.0/bin/yarn");
    plan.injected[0].mark_installed_with_version(
        "4.5.0".to_string(),
        Some(yarn_exe.clone()),
    );
    assert!(plan.injected[0].is_ready());

    // All installed, none need installation
    assert!(!plan.needs_install());
    assert_eq!(plan.runtimes_needing_install().len(), 0);

    // All have correct executable paths
    assert_eq!(plan.dependencies[0].executable, Some(node_exe));
    assert_eq!(plan.primary.executable, Some(npx_exe));
    assert_eq!(plan.injected[0].executable, Some(yarn_exe));
}
