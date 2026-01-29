//! Tests for the ConstraintsRegistry system

use rstest::rstest;
use vx_manifest::ProviderManifest;
use vx_runtime::constraints::{
    ConstraintRule, ConstraintsRegistry, DependencyConstraint, VersionPattern,
};
use vx_runtime::{get_default_constraints, init_constraints_from_manifests};

const SAMPLE_MANIFEST: &str = r#"
[provider]
name = "test-provider"

[[runtimes]]
name = "yarn"
executable = "yarn"

[[runtimes.constraints]]
when = "^1"
requires = [
  { runtime = "node", version = ">=12, <23", recommended = "20", reason = "Yarn 1.x requires Node.js 12-22" }
]

[[runtimes.constraints]]
when = "^4"
requires = [
  { runtime = "node", version = ">=18", recommended = "22" }
]

[[runtimes]]
name = "pnpm"
executable = "pnpm"

[[runtimes.constraints]]
when = "^8"
requires = [
  { runtime = "node", version = ">=16", recommended = "20" }
]

[[runtimes.constraints]]
when = "^9"
requires = [
  { runtime = "node", version = ">=18", recommended = "22" }
]
"#;

fn sample_registry() -> ConstraintsRegistry {
    ConstraintsRegistry::from_manifest_strings([("sample", SAMPLE_MANIFEST)])
        .expect("failed to build registry from manifest")
}

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
#[case("yarn", "1.22.22", "node", Some("12.0.0"), Some("23.0.0"))]
#[case("yarn", "1.0.0", "node", Some("12.0.0"), Some("23.0.0"))]
#[case("yarn", "4.0.0", "node", Some("18.0.0"), None)]
#[case("pnpm", "8.0.0", "node", Some("16.0.0"), None)]
#[case("pnpm", "9.0.0", "node", Some("18.0.0"), None)]
fn test_manifest_constraints(
    #[case] runtime: &str,
    #[case] version: &str,
    #[case] expected_dep: &str,
    #[case] expected_min: Option<&str>,
    #[case] expected_max: Option<&str>,
) {
    let registry = sample_registry();
    let deps = registry.get_constraints(runtime, version);

    assert!(
        !deps.is_empty(),
        "Expected constraints for {}@{}",
        runtime,
        version
    );

    let dep = deps.iter().find(|d| d.name == expected_dep);
    assert!(
        dep.is_some(),
        "Expected {} dependency for {}@{}",
        expected_dep,
        runtime,
        version
    );

    let dep = dep.unwrap();
    assert_eq!(dep.min_version.as_deref(), expected_min);
    assert_eq!(dep.max_version.as_deref(), expected_max);
}

#[test]
fn test_yarn_versions_have_different_constraints() {
    let registry = sample_registry();

    // Yarn 1.x has max version constraint (Node.js <23)
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
    let registry = sample_registry();
    let deps = registry.get_constraints("unknown-runtime", "1.0.0");
    assert!(deps.is_empty());
}

#[test]
fn test_custom_constraints_registry() {
    let mut registry = ConstraintsRegistry::new();

    // Register custom constraints
    registry.register(
        "my-tool",
        vec![
            ConstraintRule::new(VersionPattern::major(1)).with_constraint(
                DependencyConstraint::required("python")
                    .min("3.8.0")
                    .max("3.11.99")
                    .reason("my-tool 1.x requires Python 3.8-3.11"),
            ),
            ConstraintRule::new(VersionPattern::major(2)).with_constraint(
                DependencyConstraint::required("python")
                    .min("3.10.0")
                    .reason("my-tool 2.x requires Python 3.10+"),
            ),
        ],
    );

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
    // Initialize global registry from manifest once
    init_constraints_from_manifests([("sample", SAMPLE_MANIFEST)]).unwrap();

    let deps = get_default_constraints("yarn", "1.22.22");
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
    assert_eq!(deps_v2[0].max_version.as_deref(), None);
    assert_eq!(deps_v2[0].recommended_version.as_deref(), Some("22"));
}

#[test]
fn test_rust_cargo_constraints() {
    // Test cargo dependency on rustup
    let toml = r#"
[provider]
name = "rust"
ecosystem = "rust"

[[runtimes]]
name = "cargo"
executable = "cargo"

[[runtimes.constraints]]
when = "*"
requires = [{ runtime = "rustup", version = "*" }]
"#;

    let manifest = ProviderManifest::parse(toml).unwrap();
    let mut registry = ConstraintsRegistry::new();
    registry.load_from_manifest(&manifest);

    // cargo should require rustup
    let deps = registry.get_constraints("cargo", "1.83.0");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "rustup");
}

#[test]
fn test_rustc_constraints() {
    // Test rustc dependency on rustup
    let toml = r#"
[provider]
name = "rust"
ecosystem = "rust"

[[runtimes]]
name = "rustc"
executable = "rustc"

[[runtimes.constraints]]
when = "*"
requires = [{ runtime = "rustup", version = "*" }]
"#;

    let manifest = ProviderManifest::parse(toml).unwrap();
    let mut registry = ConstraintsRegistry::new();
    registry.load_from_manifest(&manifest);

    // rustc should require rustup
    let deps = registry.get_constraints("rustc", "1.83.0");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "rustup");
}

#[test]
fn test_npm_node_version_constraints() {
    // Test npm 9.x+ requires Node.js 14+
    let toml = r#"
[provider]
name = "node"
ecosystem = "nodejs"

[[runtimes]]
name = "npm"
executable = "npm"
bundled_with = "node"

[[runtimes.constraints]]
when = ">=9"
requires = [
    { runtime = "node", version = ">=14", recommended = "20", reason = "npm 9.x+ requires Node.js 14+" }
]

[[runtimes.constraints]]
when = "^8"
requires = [
    { runtime = "node", version = ">=12", recommended = "20" }
]
"#;

    let manifest = ProviderManifest::parse(toml).unwrap();
    let mut registry = ConstraintsRegistry::new();
    registry.load_from_manifest(&manifest);

    // npm 9.x requires Node.js 14+
    let deps_v9 = registry.get_constraints("npm", "9.0.0");
    assert_eq!(deps_v9.len(), 1);
    assert_eq!(deps_v9[0].min_version.as_deref(), Some("14.0.0"));

    // npm 8.x requires Node.js 12+
    let deps_v8 = registry.get_constraints("npm", "8.0.0");
    assert_eq!(deps_v8.len(), 1);
    assert_eq!(deps_v8[0].min_version.as_deref(), Some("12.0.0"));
}

#[test]
fn test_gofmt_go_constraints() {
    // Test gofmt bundled with go
    let toml = r#"
[provider]
name = "go"
ecosystem = "go"

[[runtimes]]
name = "gofmt"
executable = "gofmt"
bundled_with = "go"

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "go", version = "*", reason = "gofmt is bundled with go" }
]
"#;

    let manifest = ProviderManifest::parse(toml).unwrap();
    let mut registry = ConstraintsRegistry::new();
    registry.load_from_manifest(&manifest);

    // gofmt should require go
    let deps = registry.get_constraints("gofmt", "1.21.0");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "go");
}

#[test]
fn test_bunx_bun_constraints() {
    // Test bunx bundled with bun
    let toml = r#"
[provider]
name = "bun"
ecosystem = "nodejs"

[[runtimes]]
name = "bunx"
executable = "bun"
bundled_with = "bun"
command_prefix = ["x"]

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "bun", version = "*", reason = "bunx is bundled with bun" }
]
"#;

    let manifest = ProviderManifest::parse(toml).unwrap();
    let mut registry = ConstraintsRegistry::new();
    registry.load_from_manifest(&manifest);

    // bunx should require bun
    let deps = registry.get_constraints("bunx", "1.0.0");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "bun");
}

#[test]
fn test_uvx_uv_constraints() {
    // Test uvx bundled with uv
    let toml = r#"
[provider]
name = "uv"
ecosystem = "python"

[[runtimes]]
name = "uvx"
executable = "uvx"
bundled_with = "uv"

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "uv", version = "*", reason = "uvx is bundled with uv" }
]
"#;

    let manifest = ProviderManifest::parse(toml).unwrap();
    let mut registry = ConstraintsRegistry::new();
    registry.load_from_manifest(&manifest);

    // uvx should require uv
    let deps = registry.get_constraints("uvx", "0.5.0");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "uv");
}
