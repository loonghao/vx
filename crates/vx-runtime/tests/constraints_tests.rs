//! Tests for the ConstraintsRegistry system

use rstest::rstest;
use vx_runtime::constraints::{
    ConstraintRule, ConstraintsRegistry, DependencyConstraint, VersionPattern,
};

#[test]
fn test_version_pattern_major() {
    let pattern = VersionPattern::major(1);
    assert!(pattern.matches("1.0.0"));
    assert!(pattern.matches("1.22.22"));
    assert!(pattern.matches("1.99.99"));
    assert!(!pattern.matches("2.0.0"));
    assert!(!pattern.matches("0.9.0"));
}

#[test]
fn test_version_pattern_range() {
    let pattern = VersionPattern::range("2.0.0", "4.0.0");
    assert!(!pattern.matches("1.99.99"));
    assert!(pattern.matches("2.0.0"));
    assert!(pattern.matches("3.5.0"));
    assert!(!pattern.matches("4.0.0")); // max is exclusive
}

#[test]
fn test_version_pattern_all() {
    let pattern = VersionPattern::all();
    assert!(pattern.matches("1.0.0"));
    assert!(pattern.matches("99.99.99"));
    assert!(pattern.matches("0.0.1"));
}

#[rstest]
#[case("yarn", "1.22.22", "node", Some("12.0.0"), Some("22.99.99"))]
#[case("yarn", "1.0.0", "node", Some("12.0.0"), Some("22.99.99"))]
#[case("yarn", "4.0.0", "node", Some("18.0.0"), None)]
#[case("pnpm", "8.0.0", "node", Some("16.0.0"), None)]
#[case("pnpm", "9.0.0", "node", Some("18.0.0"), None)]
fn test_builtin_constraints(
    #[case] runtime: &str,
    #[case] version: &str,
    #[case] expected_dep: &str,
    #[case] expected_min: Option<&str>,
    #[case] expected_max: Option<&str>,
) {
    let registry = ConstraintsRegistry::with_builtins();
    let deps = registry.get_constraints(runtime, version);

    assert!(!deps.is_empty(), "Expected constraints for {}@{}", runtime, version);
    
    let dep = deps.iter().find(|d| d.name == expected_dep);
    assert!(dep.is_some(), "Expected {} dependency for {}@{}", expected_dep, runtime, version);
    
    let dep = dep.unwrap();
    assert_eq!(dep.min_version.as_deref(), expected_min);
    assert_eq!(dep.max_version.as_deref(), expected_max);
}

#[test]
fn test_yarn_1x_vs_4x_different_constraints() {
    let registry = ConstraintsRegistry::with_builtins();
    
    // Yarn 1.x has max version constraint (Node.js 22 max)
    let yarn1_deps = registry.get_constraints("yarn", "1.22.22");
    assert_eq!(yarn1_deps.len(), 1);
    assert!(yarn1_deps[0].max_version.is_some());
    
    // Yarn 4.x has no max version constraint
    let yarn4_deps = registry.get_constraints("yarn", "4.0.0");
    assert_eq!(yarn4_deps.len(), 1);
    assert!(yarn4_deps[0].max_version.is_none());
    
    // Different minimum versions
    assert_eq!(yarn1_deps[0].min_version.as_deref(), Some("12.0.0"));
    assert_eq!(yarn4_deps[0].min_version.as_deref(), Some("18.0.0"));
}

#[test]
fn test_no_constraints_for_unknown_runtime() {
    let registry = ConstraintsRegistry::with_builtins();
    let deps = registry.get_constraints("unknown-runtime", "1.0.0");
    assert!(deps.is_empty());
}

#[test]
fn test_custom_constraints_registry() {
    let mut registry = ConstraintsRegistry::new();
    
    // Register custom constraints
    registry.register("my-tool", vec![
        ConstraintRule::new(VersionPattern::major(1))
            .with_constraint(
                DependencyConstraint::required("python")
                    .min("3.8.0")
                    .max("3.11.99")
                    .reason("my-tool 1.x requires Python 3.8-3.11")
            ),
        ConstraintRule::new(VersionPattern::major(2))
            .with_constraint(
                DependencyConstraint::required("python")
                    .min("3.10.0")
                    .reason("my-tool 2.x requires Python 3.10+")
            ),
    ]);
    
    // Test version 1.x constraints
    let deps_v1 = registry.get_constraints("my-tool", "1.5.0");
    assert_eq!(deps_v1.len(), 1);
    assert_eq!(deps_v1[0].min_version.as_deref(), Some("3.8.0"));
    assert_eq!(deps_v1[0].max_version.as_deref(), Some("3.11.99"));
    
    // Test version 2.x constraints
    let deps_v2 = registry.get_constraints("my-tool", "2.0.0");
    assert_eq!(deps_v2.len(), 1);
    assert_eq!(deps_v2[0].min_version.as_deref(), Some("3.10.0"));
    assert!(deps_v2[0].max_version.is_none());
}

#[test]
fn test_constraint_to_runtime_dependency() {
    let constraint = DependencyConstraint::required("node")
        .min("18.0.0")
        .max("22.99.99")
        .recommended("20")
        .reason("Test reason");
    
    let dep = constraint.to_runtime_dependency();
    
    assert_eq!(dep.name, "node");
    assert_eq!(dep.min_version, Some("18.0.0".to_string()));
    assert_eq!(dep.max_version, Some("22.99.99".to_string()));
    assert_eq!(dep.recommended_version, Some("20".to_string()));
    assert_eq!(dep.reason, Some("Test reason".to_string()));
    assert!(!dep.optional);
}

#[test]
fn test_get_default_constraints() {
    // Test the global function
    let deps = vx_runtime::get_default_constraints("yarn", "1.22.22");
    assert!(!deps.is_empty());
    assert_eq!(deps[0].name, "node");
}

#[test]
fn test_load_from_manifest() {
    use vx_manifest::ProviderManifest;
    
    let toml = r#"
[provider]
name = "test-provider"

[[runtimes]]
name = "test-runtime"
executable = "test"

[[runtimes.constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=12, <23", recommended = "20", reason = "Test reason" }
]

[[runtimes.constraints]]
when = ">=2"
requires = [
    { runtime = "node", version = ">=18", recommended = "22" }
]
"#;
    
    let manifest = ProviderManifest::parse(toml).unwrap();
    let mut registry = ConstraintsRegistry::new();
    registry.load_from_manifest(&manifest);
    
    // Test v1 constraints
    let deps_v1 = registry.get_constraints("test-runtime", "1.5.0");
    assert_eq!(deps_v1.len(), 1);
    assert_eq!(deps_v1[0].name, "node");
    assert_eq!(deps_v1[0].min_version.as_deref(), Some("12.0.0"));
    assert_eq!(deps_v1[0].max_version.as_deref(), Some("23.0.0"));
    assert_eq!(deps_v1[0].recommended_version.as_deref(), Some("20"));
    assert_eq!(deps_v1[0].reason.as_deref(), Some("Test reason"));
    
    // Test v2 constraints
    let deps_v2 = registry.get_constraints("test-runtime", "2.0.0");
    assert_eq!(deps_v2.len(), 1);
    assert_eq!(deps_v2[0].name, "node");
    assert_eq!(deps_v2[0].min_version.as_deref(), Some("18.0.0"));
    assert!(deps_v2[0].max_version.is_none());
}

#[test]
fn test_manifest_version_pattern() {
    use vx_runtime::ManifestVersionPattern;
    
    // Test caret pattern
    let pattern = ManifestVersionPattern::new("^1");
    assert!(pattern.matches("1.0.0"));
    assert!(pattern.matches("1.99.99"));
    assert!(!pattern.matches("2.0.0"));
    
    // Test range pattern
    let pattern = ManifestVersionPattern::new(">=2, <4");
    assert!(!pattern.matches("1.99.99"));
    assert!(pattern.matches("2.0.0"));
    assert!(pattern.matches("3.99.99"));
    assert!(!pattern.matches("4.0.0"));
    
    // Test any pattern
    let pattern = ManifestVersionPattern::new("*");
    assert!(pattern.matches("1.0.0"));
    assert!(pattern.matches("99.99.99"));
}
