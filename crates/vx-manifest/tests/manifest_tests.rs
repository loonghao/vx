//! Tests for provider manifest parsing

use rstest::rstest;
use vx_manifest::{Ecosystem, ProviderManifest, VersionRequest};

#[test]
fn test_parse_yarn_manifest() {
    let toml = include_str!("../../vx-providers/yarn/provider.toml");
    let manifest = ProviderManifest::parse(toml).expect("Failed to parse yarn manifest");

    assert_eq!(manifest.provider.name, "yarn");
    assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::NodeJs));
    assert_eq!(manifest.runtimes.len(), 1);

    let runtime = &manifest.runtimes[0];
    assert_eq!(runtime.name, "yarn");
    assert_eq!(runtime.executable, "yarn");
    assert_eq!(runtime.aliases, vec!["yarnpkg"]);
    assert_eq!(runtime.constraints.len(), 3);
}

#[test]
fn test_parse_pnpm_manifest() {
    let toml = include_str!("../../vx-providers/pnpm/provider.toml");
    let manifest = ProviderManifest::parse(toml).expect("Failed to parse pnpm manifest");

    assert_eq!(manifest.provider.name, "pnpm");
    assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::NodeJs));
}

#[test]
fn test_parse_node_manifest() {
    let toml = include_str!("../../vx-providers/node/provider.toml");
    let manifest = ProviderManifest::parse(toml).expect("Failed to parse node manifest");

    assert_eq!(manifest.provider.name, "node");
    assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::NodeJs));
    // node, npm, npx
    assert_eq!(manifest.runtimes.len(), 3);
}

#[rstest]
#[case("20.0.0", ">=12, <23", true)]
#[case("18.0.0", ">=12", true)]
#[case("20.0.0", "<23", true)]
#[case("18.0.0", ">=20", false)]
#[case("18.0.0", ">=18", true)]
#[case("16.0.0", ">=20", false)]
fn test_constraint_matching(#[case] version: &str, #[case] constraint: &str, #[case] expected: bool) {
    let req = VersionRequest::parse(constraint);
    assert_eq!(req.satisfies(version), expected, "Version {} should {} satisfy {}", version, if expected { "" } else { "not" }, constraint);
}

#[test]
fn test_yarn_v1_constraints() {
    let toml = include_str!("../../vx-providers/yarn/provider.toml");
    let manifest = ProviderManifest::parse(toml).unwrap();
    let runtime = manifest.get_runtime("yarn").unwrap();

    // Test Yarn 1.x constraints
    let deps = runtime.get_dependencies_for_version("1.22.22");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].runtime, "node");
    assert_eq!(deps[0].version, ">=12, <23");
    assert_eq!(deps[0].recommended, Some("20".to_string()));

    // Node 20 should satisfy the constraint
    assert!(deps[0].satisfies("20.0.0"));
    // Node 10 should not satisfy
    assert!(!deps[0].satisfies("10.0.0"));
    // Node 23 should not satisfy
    assert!(!deps[0].satisfies("23.0.0"));
}

#[test]
fn test_yarn_v4_constraints() {
    let toml = include_str!("../../vx-providers/yarn/provider.toml");
    let manifest = ProviderManifest::parse(toml).unwrap();
    let runtime = manifest.get_runtime("yarn").unwrap();

    // Test Yarn 4.x constraints
    let deps = runtime.get_dependencies_for_version("4.0.0");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].runtime, "node");
    assert_eq!(deps[0].version, ">=18");
    assert_eq!(deps[0].recommended, Some("22".to_string()));

    // Node 18 should satisfy
    assert!(deps[0].satisfies("18.0.0"));
    // Node 16 should not satisfy
    assert!(!deps[0].satisfies("16.0.0"));
}

#[test]
fn test_pnpm_constraints() {
    let toml = include_str!("../../vx-providers/pnpm/provider.toml");
    let manifest = ProviderManifest::parse(toml).unwrap();
    let runtime = manifest.get_runtime("pnpm").unwrap();

    // pnpm 7.x
    let deps_v7 = runtime.get_dependencies_for_version("7.33.0");
    assert_eq!(deps_v7.len(), 1);
    assert_eq!(deps_v7[0].version, ">=14");

    // pnpm 8.x
    let deps_v8 = runtime.get_dependencies_for_version("8.15.0");
    assert_eq!(deps_v8.len(), 1);
    assert_eq!(deps_v8[0].version, ">=16");

    // pnpm 9.x
    let deps_v9 = runtime.get_dependencies_for_version("9.0.0");
    assert_eq!(deps_v9.len(), 1);
    assert_eq!(deps_v9[0].version, ">=18");
}

#[test]
fn test_get_runtime_by_alias() {
    let toml = include_str!("../../vx-providers/yarn/provider.toml");
    let manifest = ProviderManifest::parse(toml).unwrap();

    // Should find by name
    assert!(manifest.get_runtime("yarn").is_some());
    // Should find by alias
    assert!(manifest.get_runtime("yarnpkg").is_some());
    // Should not find unknown
    assert!(manifest.get_runtime("unknown").is_none());
}

#[test]
fn test_ecosystem_parsing() {
    let toml = r#"
[provider]
name = "test"
ecosystem = "python"

[[runtimes]]
name = "test"
executable = "test"
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::Python));
}

#[test]
fn test_hooks_parsing() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test"
executable = "test"

[runtimes.hooks]
pre_run = ["hook1", "hook2"]
post_install = ["hook3"]
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let hooks = manifest.runtimes[0].hooks.as_ref().unwrap();
    assert_eq!(hooks.pre_run, vec!["hook1", "hook2"]);
    assert_eq!(hooks.post_install, vec!["hook3"]);
}

#[test]
fn test_platform_config_parsing() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test"
executable = "test"

[runtimes.platforms.windows]
executable_extensions = [".cmd", ".exe"]

[runtimes.platforms.unix]
executable_extensions = []
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let platforms = manifest.runtimes[0].platforms.as_ref().unwrap();
    
    let windows = platforms.windows.as_ref().unwrap();
    assert_eq!(windows.executable_extensions, vec![".cmd", ".exe"]);
    
    let unix = platforms.unix.as_ref().unwrap();
    assert!(unix.executable_extensions.is_empty());
}

#[test]
fn test_validation_missing_provider_name() {
    let toml = r#"
[provider]
description = "test"

[[runtimes]]
name = "test"
executable = "test"
"#;
    let result = ProviderManifest::parse(toml);
    assert!(result.is_err());
}

#[test]
fn test_validation_missing_runtime_executable() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test"
"#;
    let result = ProviderManifest::parse(toml);
    assert!(result.is_err());
}
