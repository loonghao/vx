//! Tests for EnsureStage
//!
//! These tests cover the EnsureStage behavior that can be tested without
//! a real InstallationManager (i.e., cases where installation is not triggered).
//!
//! For the installation + version propagation behavior, see `plan_tests.rs`
//! which tests `mark_installed_with_version` directly — the method that
//! EnsureStage calls after successful installation.

use std::path::PathBuf;
use vx_resolver::{
    EnsureError, EnsureStage, ExecutionConfig, ExecutionPlan, PlannedRuntime, Resolver,
    ResolverConfig, RuntimeMap, Stage,
};

fn create_ensure_stage<'a>(
    resolver: &'a Resolver,
    config: &'a ResolverConfig,
) -> EnsureStage<'a> {
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
    let plan =
        ExecutionPlan::new(primary, ExecutionConfig::default()).with_injected(unsupported);

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
