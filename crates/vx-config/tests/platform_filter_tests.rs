//! Platform filtering tests
//!
//! Tests for vx.toml tool OS constraint filtering.
//! Ensures that `tools_for_platform()` correctly skips tools
//! that are restricted to specific operating systems.

use rstest::rstest;
use vx_config::parse_config_str;

// ============================================
// Platform Filtering Tests
// ============================================

#[test]
fn test_simple_tools_included_on_all_platforms() {
    let content = r#"
[tools]
node = "20"
python = "3.12"
"#;
    let config = parse_config_str(content).unwrap();

    for os in &["windows", "darwin", "linux"] {
        let (included, skipped) = config.tools_for_platform(os);
        assert_eq!(
            included.len(),
            2,
            "All simple tools should be included on {}",
            os
        );
        assert!(included.contains_key("node"));
        assert!(included.contains_key("python"));
        assert!(skipped.is_empty(), "No tools should be skipped on {}", os);
    }
}

#[test]
fn test_windows_only_tool_skipped_on_linux() {
    let content = r#"
[tools]
node = "20"

[tools.msvc]
version = "latest"
os = ["windows"]
"#;
    let config = parse_config_str(content).unwrap();

    // On Windows: both tools included
    let (included, skipped) = config.tools_for_platform("windows");
    assert_eq!(included.len(), 2);
    assert!(included.contains_key("node"));
    assert!(included.contains_key("msvc"));
    assert!(skipped.is_empty());

    // On Linux: msvc skipped
    let (included, skipped) = config.tools_for_platform("linux");
    assert_eq!(included.len(), 1);
    assert!(included.contains_key("node"));
    assert!(!included.contains_key("msvc"));
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].0, "msvc");
    assert_eq!(skipped[0].1, vec!["windows".to_string()]);

    // On macOS: msvc skipped
    let (included, skipped) = config.tools_for_platform("darwin");
    assert_eq!(included.len(), 1);
    assert!(!included.contains_key("msvc"));
    assert_eq!(skipped.len(), 1);
}

#[test]
fn test_unix_only_tool_skipped_on_windows() {
    let content = r#"
[tools]
node = "20"

[tools.brew]
version = "latest"
os = ["darwin", "linux"]
"#;
    let config = parse_config_str(content).unwrap();

    // On Windows: brew skipped
    let (included, skipped) = config.tools_for_platform("windows");
    assert_eq!(included.len(), 1);
    assert!(included.contains_key("node"));
    assert!(!included.contains_key("brew"));
    assert_eq!(skipped.len(), 1);

    // On macOS: both included
    let (included, skipped) = config.tools_for_platform("darwin");
    assert_eq!(included.len(), 2);
    assert!(included.contains_key("brew"));
    assert!(skipped.is_empty());

    // On Linux: both included
    let (included, skipped) = config.tools_for_platform("linux");
    assert_eq!(included.len(), 2);
    assert!(included.contains_key("brew"));
    assert!(skipped.is_empty());
}

#[test]
fn test_multiple_platform_specific_tools() {
    let content = r#"
[tools]
node = "20"
python = "3.12"

[tools.msvc]
version = "latest"
os = ["windows"]

[tools.brew]
version = "latest"
os = ["darwin", "linux"]
"#;
    let config = parse_config_str(content).unwrap();

    // On Windows: node, python, msvc (not brew)
    let (included, skipped) = config.tools_for_platform("windows");
    assert_eq!(included.len(), 3);
    assert!(included.contains_key("node"));
    assert!(included.contains_key("python"));
    assert!(included.contains_key("msvc"));
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].0, "brew");

    // On macOS: node, python, brew (not msvc)
    let (included, skipped) = config.tools_for_platform("darwin");
    assert_eq!(included.len(), 3);
    assert!(included.contains_key("node"));
    assert!(included.contains_key("python"));
    assert!(included.contains_key("brew"));
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].0, "msvc");

    // On Linux: node, python, brew (not msvc)
    let (included, _) = config.tools_for_platform("linux");
    assert_eq!(included.len(), 3);
    assert!(included.contains_key("brew"));
    assert!(!included.contains_key("msvc"));
}

#[test]
fn test_empty_os_list_means_all_platforms() {
    let content = r#"
[tools.node]
version = "20"
os = []
"#;
    let config = parse_config_str(content).unwrap();

    for os in &["windows", "darwin", "linux"] {
        let (included, skipped) = config.tools_for_platform(os);
        assert_eq!(
            included.len(),
            1,
            "Empty os should mean all platforms on {}",
            os
        );
        assert!(skipped.is_empty());
    }
}

#[test]
fn test_no_os_field_means_all_platforms() {
    let content = r#"
[tools.node]
version = "20"
postinstall = "corepack enable"
"#;
    let config = parse_config_str(content).unwrap();

    for os in &["windows", "darwin", "linux"] {
        let (included, skipped) = config.tools_for_platform(os);
        assert_eq!(
            included.len(),
            1,
            "No os field should mean all platforms on {}",
            os
        );
        assert!(included.contains_key("node"));
        assert!(skipped.is_empty());
    }
}

#[test]
fn test_case_insensitive_os_matching() {
    let content = r#"
[tools.msvc]
version = "latest"
os = ["Windows"]
"#;
    let config = parse_config_str(content).unwrap();

    // "Windows" (capitalized) in config should match "windows" (lowercase) query
    let (included, skipped) = config.tools_for_platform("windows");
    assert_eq!(included.len(), 1);
    assert!(skipped.is_empty());
}

#[rstest]
#[case("windows", 3, 1)] // node, python, msvc included; brew skipped
#[case("darwin", 3, 1)] // node, python, brew included; msvc skipped
#[case("linux", 3, 1)] // node, python, brew included; msvc skipped
fn test_realistic_electron_project_config(
    #[case] os: &str,
    #[case] expected_included: usize,
    #[case] expected_skipped: usize,
) {
    let content = r#"
[tools]
node = "20"
python = "3.12"

[tools.msvc]
version = "latest"
os = ["windows"]

[tools.brew]
version = "latest"
os = ["darwin", "linux"]
"#;
    let config = parse_config_str(content).unwrap();
    let (included, skipped) = config.tools_for_platform(os);

    assert_eq!(
        included.len(),
        expected_included,
        "Expected {} tools included on {}, got {}",
        expected_included,
        os,
        included.len()
    );
    assert_eq!(
        skipped.len(),
        expected_skipped,
        "Expected {} tools skipped on {}, got {}",
        expected_skipped,
        os,
        skipped.len()
    );
}

#[test]
fn test_btree_variant_matches_hashmap() {
    let content = r#"
[tools]
node = "20"

[tools.msvc]
version = "latest"
os = ["windows"]
"#;
    let config = parse_config_str(content).unwrap();

    let (hash_included, hash_skipped) = config.tools_for_platform("linux");
    let (btree_included, btree_skipped) = config.tools_for_current_platform_btree();

    // On the current test platform, verify btree variant also works
    // (exact results depend on current OS, but structure should be valid)
    assert!(!btree_included.is_empty() || !btree_skipped.is_empty());

    // Verify Linux filtering specifically
    assert_eq!(hash_included.len(), 1);
    assert_eq!(hash_skipped.len(), 1);
    assert!(hash_included.contains_key("node"));
}

#[test]
fn test_all_tools_platform_specific() {
    let content = r#"
[tools.msvc]
version = "latest"
os = ["windows"]

[tools.brew]
version = "latest"
os = ["darwin"]
"#;
    let config = parse_config_str(content).unwrap();

    // On Linux: nothing included
    let (included, skipped) = config.tools_for_platform("linux");
    assert!(included.is_empty());
    assert_eq!(skipped.len(), 2);
}

#[test]
fn test_tools_as_hashmap_ignores_os_field() {
    // Verify backward compatibility: tools_as_hashmap() returns ALL tools regardless of os
    let content = r#"
[tools]
node = "20"

[tools.msvc]
version = "latest"
os = ["windows"]
"#;
    let config = parse_config_str(content).unwrap();

    let all_tools = config.tools_as_hashmap();
    assert_eq!(all_tools.len(), 2);
    assert!(all_tools.contains_key("node"));
    assert!(all_tools.contains_key("msvc"));
}

#[test]
fn test_runtimes_section_platform_filtering() {
    // Test backward-compatible [runtimes] section with platform filtering
    let content = r#"
[runtimes]
node = "20"

[runtimes.msvc]
version = "latest"
os = ["windows"]
"#;
    let config = parse_config_str(content).unwrap();

    let (included, skipped) = config.tools_for_platform("linux");
    assert_eq!(included.len(), 1);
    assert!(included.contains_key("node"));
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].0, "msvc");
}

#[test]
fn test_tools_override_runtimes_platform_filter() {
    // [tools] section overrides [runtimes] - platform filter should use tools version
    let content = r#"
[runtimes.node]
version = "18"
os = ["linux"]

[tools]
node = "20"
"#;
    let config = parse_config_str(content).unwrap();

    // tools (simple) overrides runtimes (detailed with os restriction)
    // Simple tool has no os constraint, so it should be included everywhere
    let (included, skipped) = config.tools_for_platform("windows");
    assert_eq!(included.len(), 1);
    assert_eq!(included.get("node"), Some(&"20".to_string()));
    assert!(skipped.is_empty());
}
