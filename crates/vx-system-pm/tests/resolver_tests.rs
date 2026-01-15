//! Tests for SystemDependencyResolver

use vx_system_pm::{SystemDepType, SystemDependency, SystemDependencyResolver};

#[tokio::test]
async fn test_resolver_creation() {
    let _resolver = SystemDependencyResolver::new();
    // Just ensure it creates without panicking
    assert!(true);
}

#[tokio::test]
async fn test_resolve_empty_deps() {
    let mut resolver = SystemDependencyResolver::new();
    let deps: Vec<SystemDependency> = vec![];

    let result = resolver.resolve(&deps).await.unwrap();

    assert!(result.to_install.is_empty());
    assert!(result.satisfied.is_empty());
    assert!(result.unresolved.is_empty());
}

#[tokio::test]
async fn test_resolve_platform_filtered() {
    let mut resolver = SystemDependencyResolver::new();

    // Create a dependency for a different platform
    let deps = vec![
        SystemDependency::new(SystemDepType::Package, "test-package")
            .with_platforms(vec!["nonexistent-platform".to_string()]),
    ];

    let result = resolver.resolve(&deps).await.unwrap();

    // Should be filtered out due to platform mismatch
    assert!(result.to_install.is_empty());
    assert!(result.satisfied.is_empty());
    assert!(result.unresolved.is_empty());
}

#[test]
fn test_system_dependency_builder() {
    let dep = SystemDependency::new(SystemDepType::VcRedist, "vcredist140")
        .with_version(">=14.0")
        .with_reason("Required for C++ runtime")
        .with_platforms(vec!["windows".to_string()])
        .optional();

    assert_eq!(dep.id, "vcredist140");
    assert_eq!(dep.dep_type, SystemDepType::VcRedist);
    assert_eq!(dep.version, Some(">=14.0".to_string()));
    assert_eq!(dep.reason, Some("Required for C++ runtime".to_string()));
    assert!(dep.optional);
}

#[test]
fn test_system_dep_type_display() {
    assert_eq!(format!("{}", SystemDepType::WindowsKb), "Windows KB");
    assert_eq!(
        format!("{}", SystemDepType::VcRedist),
        "VC++ Redistributable"
    );
    assert_eq!(format!("{}", SystemDepType::DotNet), ".NET");
    assert_eq!(format!("{}", SystemDepType::Package), "Package");
}
