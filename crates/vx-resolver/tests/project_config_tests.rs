//! Tests for ProjectToolsConfig, especially companion tools injection
//! and version priority rules (vx.lock > vx.toml)

use rstest::rstest;
use std::collections::HashMap;
use vx_resolver::ProjectToolsConfig;

/// Helper to create a ProjectToolsConfig from a list of (tool, version) pairs
fn config_from(tools: &[(&str, &str)]) -> ProjectToolsConfig {
    let map: HashMap<String, String> = tools
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    ProjectToolsConfig::from_tools(map)
}

/// Helper to create a ProjectToolsConfig with both tools and locked versions
fn config_with_locked(tools: &[(&str, &str)], locked: &[(&str, &str)]) -> ProjectToolsConfig {
    let tools_map: HashMap<String, String> = tools
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let locked_map: HashMap<String, String> = locked
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    ProjectToolsConfig::from_tools_with_locked(tools_map, locked_map)
}

// =============================================================================
// get_companion_tools tests
// =============================================================================

#[rstest]
fn test_companion_tools_msvc_with_node() {
    // When running `vx node`, msvc should be a companion tool
    let config = config_from(&[("node", "22"), ("msvc", "14.42")]);
    let companions = config.get_companion_tools("node");
    assert_eq!(companions.len(), 1);
    assert_eq!(companions[0].0, "msvc");
    assert_eq!(companions[0].1, "14.42");
}

#[rstest]
fn test_companion_tools_excludes_self() {
    // The primary runtime should never be in the companion list
    let config = config_from(&[("node", "22"), ("msvc", "14.42")]);
    let companions = config.get_companion_tools("node");
    assert!(!companions.iter().any(|(name, _)| *name == "node"));
}

#[rstest]
fn test_companion_tools_excludes_bundled_tools() {
    // npm is bundled with node, so when running node, npm should not be a companion
    let config = config_from(&[("node", "22"), ("npm", "10"), ("msvc", "14.42")]);
    let companions = config.get_companion_tools("node");
    assert!(!companions.iter().any(|(name, _)| *name == "npm"));
    assert!(companions.iter().any(|(name, _)| *name == "msvc"));
}

#[rstest]
fn test_companion_tools_from_bundled_tool_perspective() {
    // When running npm, node should NOT be a companion (npm is bundled with node)
    let config = config_from(&[("node", "22"), ("npm", "10"), ("msvc", "14.42")]);
    let companions = config.get_companion_tools("npm");
    assert!(!companions.iter().any(|(name, _)| *name == "node"));
    assert!(companions.iter().any(|(name, _)| *name == "msvc"));
}

#[rstest]
fn test_companion_tools_when_running_msvc() {
    // When running msvc directly, node should be a companion
    let config = config_from(&[("node", "22"), ("msvc", "14.42")]);
    let companions = config.get_companion_tools("msvc");
    assert_eq!(companions.len(), 1);
    assert_eq!(companions[0].0, "node");
}

#[rstest]
fn test_companion_tools_empty_when_only_primary() {
    // No companions when only one tool is configured
    let config = config_from(&[("node", "22")]);
    let companions = config.get_companion_tools("node");
    assert!(companions.is_empty());
}

#[rstest]
fn test_companion_tools_multiple_companions() {
    // Multiple companion tools
    let config = config_from(&[("node", "22"), ("msvc", "14.42"), ("rust", "1.80")]);
    let mut companions = config.get_companion_tools("node");
    companions.sort_by_key(|(name, _)| name.to_string());
    assert_eq!(companions.len(), 2);
    assert_eq!(companions[0].0, "msvc");
    assert_eq!(companions[1].0, "rust");
}

#[rstest]
fn test_companion_tools_independent_package_managers_included() {
    // pnpm is NOT bundled with node, so it should be a companion
    let config = config_from(&[("node", "22"), ("pnpm", "9"), ("msvc", "14.42")]);
    let companions = config.get_companion_tools("node");
    assert!(companions.iter().any(|(name, _)| *name == "pnpm"));
    assert!(companions.iter().any(|(name, _)| *name == "msvc"));
}

#[rstest]
fn test_companion_tools_rust_ecosystem() {
    // cargo is bundled with rust; when running rust, cargo shouldn't be companion
    let config = config_from(&[("rust", "1.80"), ("cargo", "1.80"), ("node", "22")]);
    let companions = config.get_companion_tools("rust");
    assert!(!companions.iter().any(|(name, _)| *name == "cargo"));
    assert!(companions.iter().any(|(name, _)| *name == "node"));
}

// =============================================================================
// get_version_with_fallback tests (existing behavior verification)
// =============================================================================

#[rstest]
fn test_version_direct_lookup() {
    let config = config_from(&[("node", "22"), ("msvc", "14.42")]);
    assert_eq!(config.get_version("node"), Some("22"));
    assert_eq!(config.get_version("msvc"), Some("14.42"));
    assert_eq!(config.get_version("go"), None);
}

#[rstest]
fn test_version_with_fallback_bundled() {
    let config = config_from(&[("node", "22")]);
    // npm is bundled with node, should fallback to node's version
    assert_eq!(config.get_version_with_fallback("npm"), Some("22"));
    // pnpm is independent, should NOT fallback
    assert_eq!(config.get_version_with_fallback("pnpm"), None);
}

// =============================================================================
// Version Priority tests: vx.lock > vx.toml
// =============================================================================

#[rstest]
fn test_version_priority_lock_over_config() {
    // When both vx.lock and vx.toml have a version for the same tool,
    // vx.lock should take priority
    let config = config_with_locked(
        &[("node", "22")],      // vx.toml: node = "22"
        &[("node", "20.18.0")], // vx.lock: node = "20.18.0"
    );

    // get_version should return the locked version
    assert_eq!(config.get_version("node"), Some("20.18.0"));
    assert!(config.is_locked("node"));
}

#[rstest]
fn test_version_priority_lock_missing_uses_config() {
    // When vx.lock doesn't have a tool, vx.toml version is used
    let config = config_with_locked(
        &[("node", "22"), ("go", "1.21")], // vx.toml has both
        &[("node", "20.18.0")],            // vx.lock only has node
    );

    // node: locked version
    assert_eq!(config.get_version("node"), Some("20.18.0"));
    assert!(config.is_locked("node"));

    // go: config version (not locked)
    assert_eq!(config.get_version("go"), Some("1.21"));
    assert!(!config.is_locked("go"));
}

#[rstest]
fn test_version_priority_only_config() {
    // When there's no vx.lock, vx.toml version is used
    let config = config_from(&[("node", "22"), ("go", "1.21")]);

    assert_eq!(config.get_version("node"), Some("22"));
    assert_eq!(config.get_version("go"), Some("1.21"));
    assert!(!config.is_locked("node"));
    assert!(!config.is_locked("go"));
}

#[rstest]
fn test_version_priority_only_locked() {
    // When vx.toml is empty but vx.lock has versions
    let config = config_with_locked(
        &[],                                      // vx.toml has no tools
        &[("node", "20.18.0"), ("go", "1.21.5")], // vx.lock has versions
    );

    assert_eq!(config.get_version("node"), Some("20.18.0"));
    assert_eq!(config.get_version("go"), Some("1.21.5"));
    assert!(config.is_locked("node"));
    assert!(config.is_locked("go"));
}

#[rstest]
fn test_version_priority_unknown_tool() {
    let config = config_with_locked(&[("node", "22")], &[("node", "20.18.0")]);

    // Unknown tool returns None
    assert_eq!(config.get_version("unknown-tool"), None);
    assert!(!config.is_locked("unknown-tool"));
}

#[rstest]
fn test_version_priority_with_fallback_respects_lock() {
    // get_version_with_fallback should also respect lock priority
    let config = config_with_locked(
        &[("node", "22")],      // vx.toml: node = "22"
        &[("node", "20.18.0")], // vx.lock: node = "20.18.0"
    );

    // npm falls back to node's version, which should be the locked version
    assert_eq!(config.get_version_with_fallback("npm"), Some("20.18.0"));
    assert_eq!(config.get_version_with_fallback("npx"), Some("20.18.0"));
}

#[rstest]
fn test_version_priority_with_fallback_lock_for_different_tool() {
    // When falling back, check locked version of primary runtime
    let config = config_with_locked(
        &[("node", "22"), ("python", "3.12")], // vx.toml
        &[("node", "20.18.0")],                // vx.lock only has node
    );

    // npm falls back to node's LOCKED version
    assert_eq!(config.get_version_with_fallback("npm"), Some("20.18.0"));

    // pip falls back to python's CONFIG version (not locked)
    assert_eq!(config.get_version_with_fallback("pip"), Some("3.12"));
}

#[rstest]
fn test_locked_tool_names() {
    let config = config_with_locked(
        &[("node", "22"), ("go", "1.21")],
        &[("node", "20.18.0"), ("go", "1.21.5")],
    );

    let mut locked = config.locked_tool_names();
    locked.sort();

    assert_eq!(locked, vec!["go", "node"]);
}
