//! Tests for EnsureStage
//!
//! These tests cover the EnsureStage behavior that can be tested without
//! a real InstallationManager (i.e., cases where installation is not triggered).
//!
//! For the installation + version/executable propagation behavior, see `plan_tests.rs`
//! which tests `mark_installed_with_version` directly — the method that
//! EnsureStage calls after successful installation.
//!
//! The key design fix: EnsureStage now uses `InstallResult.executable_path`
//! directly instead of re-resolving via filesystem scan (which was unreliable,
//! especially on Windows CI). See plan_tests.rs for executable propagation tests.

use std::path::PathBuf;
use vx_resolver::{
    EnsureError, EnsureStage, ExecutionConfig, ExecutionPlan, InstallStatus, PlannedRuntime,
    Resolver, ResolverConfig, RuntimeMap, Stage,
};

fn create_ensure_stage<'a>(resolver: &'a Resolver, config: &'a ResolverConfig) -> EnsureStage<'a> {
    EnsureStage::new(resolver, config, None, None)
}

#[test]
fn test_ensure_stage_creation() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
    let _stage = create_ensure_stage(&resolver, &config);
}

#[tokio::test]
async fn test_ensure_stage_already_installed() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
    let stage = create_ensure_stage(&resolver, &config);

    let primary = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/usr/local/bin/node"),
    );
    let plan = ExecutionPlan::new(primary, ExecutionConfig::default());

    let result: Result<ExecutionPlan, EnsureError> = stage.execute(plan).await;
    assert!(result.is_ok());
    let plan = result.unwrap();
    assert!(!plan.needs_install());
}

#[tokio::test]
async fn test_ensure_stage_auto_install_disabled() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
    let stage = create_ensure_stage(&resolver, &config);

    let primary = PlannedRuntime::needs_install("node", "20.0.0".to_string());
    let exec_config = ExecutionConfig {
        auto_install: false,
        ..Default::default()
    };
    let plan = ExecutionPlan::new(primary, exec_config);

    let result: Result<ExecutionPlan, EnsureError> = stage.execute(plan).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, EnsureError::AutoInstallDisabled { .. }));
}

#[tokio::test]
async fn test_ensure_stage_auto_install_disabled_reports_all_missing() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
    let stage = create_ensure_stage(&resolver, &config);

    let primary = PlannedRuntime::needs_install("npm", "10.0.0".to_string());
    let dep = PlannedRuntime::needs_install("node", "20.0.0".to_string());
    let exec_config = ExecutionConfig {
        auto_install: false,
        ..Default::default()
    };
    let plan = ExecutionPlan::new(primary, exec_config).with_dependency(dep);

    let result: Result<ExecutionPlan, EnsureError> = stage.execute(plan).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        EnsureError::AutoInstallDisabled { runtime, .. } => {
            // Should report all missing runtimes
            assert!(runtime.contains("node"));
            assert!(runtime.contains("npm"));
        }
        other => panic!("Expected AutoInstallDisabled, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_ensure_stage_platform_unsupported_logged() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
    let stage = create_ensure_stage(&resolver, &config);

    let primary = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/usr/local/bin/node"),
    );
    let unsupported = PlannedRuntime::unsupported("msvc", "Windows only".to_string());
    let plan = ExecutionPlan::new(primary, ExecutionConfig::default()).with_injected(unsupported);

    // Should succeed (unsupported injected dep is just a warning)
    let result: Result<ExecutionPlan, EnsureError> = stage.execute(plan).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ensure_stage_no_install_needed_returns_plan_unchanged() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
    let stage = create_ensure_stage(&resolver, &config);

    let primary = PlannedRuntime::installed(
        "go",
        "1.21.0".to_string(),
        PathBuf::from("/usr/local/go/bin/go"),
    );
    let dep = PlannedRuntime::installed(
        "gofmt",
        "1.21.0".to_string(),
        PathBuf::from("/usr/local/go/bin/gofmt"),
    );

    let plan = ExecutionPlan::new(primary, ExecutionConfig::default()).with_dependency(dep);

    let result: Result<ExecutionPlan, EnsureError> = stage.execute(plan).await;
    let result = result.unwrap();
    assert_eq!(result.primary.name, "go");
    assert_eq!(result.primary.version_string(), Some("1.21.0"));
    assert_eq!(result.dependencies.len(), 1);
    assert_eq!(result.dependencies[0].name, "gofmt");
}

// ============================================================
// Tests verifying that already-installed runtimes pass through
// the EnsureStage with their executable paths intact.
// ============================================================

#[tokio::test]
async fn test_ensure_stage_preserves_executable_path_for_installed_primary() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
    let stage = create_ensure_stage(&resolver, &config);

    let exe_path = PathBuf::from("/home/user/.vx/store/uv/0.6.12/uv");
    let primary = PlannedRuntime::installed("uv", "0.6.12".to_string(), exe_path.clone());
    let plan = ExecutionPlan::new(primary, ExecutionConfig::default());

    let result = stage.execute(plan).await.unwrap();

    // Executable should be preserved through EnsureStage
    assert_eq!(result.primary.executable, Some(exe_path));
    assert!(result.primary.is_ready());
    assert_eq!(result.primary.status, InstallStatus::Installed);
}

#[tokio::test]
async fn test_ensure_stage_preserves_executable_path_for_dependencies() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
    let stage = create_ensure_stage(&resolver, &config);

    let primary_exe = PathBuf::from("/home/user/.vx/store/node/20.0.0/bin/npm");
    let dep_exe = PathBuf::from("/home/user/.vx/store/node/20.0.0/bin/node");

    let primary = PlannedRuntime::installed("npm", "10.0.0".to_string(), primary_exe.clone());
    let dep = PlannedRuntime::installed("node", "20.0.0".to_string(), dep_exe.clone());

    let plan = ExecutionPlan::new(primary, ExecutionConfig::default()).with_dependency(dep);

    let result = stage.execute(plan).await.unwrap();

    assert_eq!(result.primary.executable, Some(primary_exe));
    assert_eq!(result.dependencies[0].executable, Some(dep_exe));
    assert!(result.primary.is_ready());
    assert!(result.dependencies[0].is_ready());
}

#[tokio::test]
async fn test_ensure_stage_preserves_executable_path_for_injected() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
    let stage = create_ensure_stage(&resolver, &config);

    let primary_exe = PathBuf::from("/home/user/.vx/store/node/20.0.0/bin/node");
    let injected_exe = PathBuf::from("/home/user/.vx/store/yarn/4.5.0/bin/yarn");

    let primary = PlannedRuntime::installed("node", "20.0.0".to_string(), primary_exe.clone());
    let injected = PlannedRuntime::installed("yarn", "4.5.0".to_string(), injected_exe.clone());

    let plan = ExecutionPlan::new(primary, ExecutionConfig::default()).with_injected(injected);

    let result = stage.execute(plan).await.unwrap();

    assert_eq!(result.primary.executable, Some(primary_exe));
    assert_eq!(result.injected[0].executable, Some(injected_exe));
    assert!(result.injected[0].is_ready());
}

/// Full multi-runtime scenario: all installed, all have executables, all pass through.
#[tokio::test]
async fn test_ensure_stage_full_plan_all_installed() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
    let stage = create_ensure_stage(&resolver, &config);

    let primary = PlannedRuntime::installed(
        "npx",
        "10.0.0".to_string(),
        PathBuf::from("/home/user/.vx/store/node/20.0.0/bin/npx"),
    );
    let dep = PlannedRuntime::installed(
        "node",
        "20.0.0".to_string(),
        PathBuf::from("/home/user/.vx/store/node/20.0.0/bin/node"),
    );
    let injected = PlannedRuntime::installed(
        "yarn",
        "4.5.0".to_string(),
        PathBuf::from("/home/user/.vx/store/yarn/4.5.0/bin/yarn"),
    );

    let plan = ExecutionPlan::new(primary, ExecutionConfig::default())
        .with_dependency(dep)
        .with_injected(injected);

    let result = stage.execute(plan).await.unwrap();

    // All runtimes should be ready with executables intact
    assert!(result.primary.is_ready());
    assert!(result.dependencies[0].is_ready());
    assert!(result.injected[0].is_ready());
    assert!(!result.needs_install());
}
