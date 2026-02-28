//! Tests for StarMetadata parsing

use vx_star_metadata::StarMetadata;

// RFC 0038 v4: function-based format
const SAMPLE_STAR: &str = r#"
def name():
    return "msvc"

def description():
    return "Microsoft Visual C++ Build Tools"

def homepage():
    return "https://visualstudio.microsoft.com/visual-cpp-build-tools/"

def ecosystem():
    return "system"

def platforms():
    return {"os": ["windows"]}

runtimes = [
    {
        "name":             "msvc",
        "executable":       "cl",
        "description":      "Microsoft Visual C++ compiler",
        "aliases":          ["cl", "vs-build-tools", "msvc-tools"],
        "priority":         100,
        "auto_installable": True,
        "platform_constraint": {"os": ["windows"]},
    },
    {
        "name":             "nmake",
        "executable":       "nmake",
        "description":      "Microsoft Program Maintenance Utility",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
    },
]
"#;

// RFC 0038 v5: top-level variable format
const SAMPLE_STAR_V5: &str = r#"
name        = "node"
description = "Node.js - JavaScript runtime built on Chrome's V8 engine"
homepage    = "https://nodejs.org"
repository  = "https://github.com/nodejs/node"
ecosystem   = "nodejs"

runtimes = [
    {
        "name":       "node",
        "executable": "node",
        "aliases":    ["nodejs"],
        "priority":   100,
    },
    {"name": "npm",  "executable": "npm",  "bundled_with": "node"},
    {"name": "npx",  "executable": "npx",  "bundled_with": "node"},
]
"#;

// RFC 0038: runtime_def() / bundled_runtime_def() function call format
const SAMPLE_STAR_FUNC_CALLS: &str = r#"
name        = "node"
description = "Node.js JavaScript runtime"
ecosystem   = "nodejs"

runtimes = [
    runtime_def("node",
        aliases = ["nodejs"],
    ),
    bundled_runtime_def("npm",  bundled_with = "node"),
    bundled_runtime_def("npx",  bundled_with = "node"),
]
"#;

// Test parsing of 7zip-style runtime_def with extra whitespace
const SAMPLE_STAR_7ZIP_STYLE: &str = r#"
name        = "7zip"
description = "7-Zip - High compression ratio file archiver"
ecosystem   = "system"

runtimes = [
    runtime_def("7zip",
        aliases      = ["7z", "7za", "7zz"],
        system_paths = [
            "C:/Program Files/7-Zip",
            "/usr/bin",
        ],
    ),
]
"#;

fn vx_provider_node_star() -> &'static str {
    // Inline a minimal version of node's provider.star for testing
    r#"
load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "fetch_versions_from_api",
     "system_permissions",
     "bin_subdir_layout", "bin_subdir_env", "bin_subdir_execute_path",
     "post_extract_permissions", "pre_run_ensure_deps")
load("@vx//stdlib:env.star", "env_prepend")

name        = "node"
description = "Node.js - JavaScript runtime built on Chrome's V8 engine"
homepage    = "https://nodejs.org"
repository  = "https://github.com/nodejs/node"
license     = "MIT"
ecosystem   = "nodejs"
aliases     = ["nodejs"]

runtimes = [
    runtime_def("node",
        aliases = ["nodejs"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "^v?\\d+\\.\\d+\\.\\d+"},
        ],
    ),
    bundled_runtime_def("npm",  bundled_with = "node",
        version_pattern = "^\\d+\\.\\d+\\.\\d+"),
    bundled_runtime_def("npx",  bundled_with = "node",
        version_pattern = "^\\d+\\.\\d+\\.\\d+"),
]
"#
}

#[test]
fn test_parse_name() {
    let meta = StarMetadata::parse(SAMPLE_STAR);
    assert_eq!(meta.name, Some("msvc".to_string()));
}

#[test]
fn test_parse_description() {
    let meta = StarMetadata::parse(SAMPLE_STAR);
    assert_eq!(
        meta.description,
        Some("Microsoft Visual C++ Build Tools".to_string())
    );
}

#[test]
fn test_parse_homepage() {
    let meta = StarMetadata::parse(SAMPLE_STAR);
    assert_eq!(
        meta.homepage,
        Some("https://visualstudio.microsoft.com/visual-cpp-build-tools/".to_string())
    );
}

#[test]
fn test_parse_ecosystem() {
    let meta = StarMetadata::parse(SAMPLE_STAR);
    assert_eq!(meta.ecosystem, Some("system".to_string()));
}

#[test]
fn test_parse_platforms() {
    let meta = StarMetadata::parse(SAMPLE_STAR);
    assert_eq!(meta.platforms, Some(vec!["windows".to_string()]));
}

#[test]
fn test_parse_runtimes_count() {
    let meta = StarMetadata::parse(SAMPLE_STAR);
    assert_eq!(meta.runtimes.len(), 2);
}

#[test]
fn test_parse_runtime_name() {
    let meta = StarMetadata::parse(SAMPLE_STAR);
    assert_eq!(meta.runtimes[0].name, Some("msvc".to_string()));
    assert_eq!(meta.runtimes[1].name, Some("nmake".to_string()));
}

#[test]
fn test_parse_runtime_aliases() {
    let meta = StarMetadata::parse(SAMPLE_STAR);
    assert_eq!(
        meta.runtimes[0].aliases,
        vec!["cl", "vs-build-tools", "msvc-tools"]
    );
}

#[test]
fn test_parse_runtime_auto_installable() {
    let meta = StarMetadata::parse(SAMPLE_STAR);
    assert_eq!(meta.runtimes[0].auto_installable, Some(true));
    assert_eq!(meta.runtimes[1].auto_installable, Some(false));
}

#[test]
fn test_find_runtime_by_alias() {
    let meta = StarMetadata::parse(SAMPLE_STAR);
    let rt = meta.find_runtime("cl");
    assert!(rt.is_some());
    assert_eq!(rt.unwrap().name, Some("msvc".to_string()));
}

#[test]
fn test_name_or_fallback() {
    let meta = StarMetadata::default();
    assert_eq!(meta.name_or("fallback"), "fallback");
}

// RFC 0038 v5 tests

#[test]
fn test_parse_v5_name() {
    let meta = StarMetadata::parse(SAMPLE_STAR_V5);
    assert_eq!(meta.name, Some("node".to_string()));
}

#[test]
fn test_parse_v5_description() {
    let meta = StarMetadata::parse(SAMPLE_STAR_V5);
    assert_eq!(
        meta.description,
        Some("Node.js - JavaScript runtime built on Chrome's V8 engine".to_string())
    );
}

#[test]
fn test_parse_v5_ecosystem() {
    let meta = StarMetadata::parse(SAMPLE_STAR_V5);
    assert_eq!(meta.ecosystem, Some("nodejs".to_string()));
}

#[test]
fn test_parse_v5_runtimes() {
    let meta = StarMetadata::parse(SAMPLE_STAR_V5);
    assert_eq!(meta.runtimes.len(), 3);
    assert_eq!(meta.runtimes[0].name, Some("node".to_string()));
    assert_eq!(meta.runtimes[0].aliases, vec!["nodejs"]);
    assert_eq!(meta.runtimes[1].name, Some("npm".to_string()));
    assert_eq!(meta.runtimes[1].bundled_with, Some("node".to_string()));
}

// RFC 0038: runtime_def() / bundled_runtime_def() function call format tests

#[test]
fn test_parse_runtime_def_calls_count() {
    let meta = StarMetadata::parse(SAMPLE_STAR_FUNC_CALLS);
    assert_eq!(
        meta.runtimes.len(),
        3,
        "Expected 3 runtimes from runtime_def/bundled_runtime_def calls"
    );
}

#[test]
fn test_parse_runtime_def_name() {
    let meta = StarMetadata::parse(SAMPLE_STAR_FUNC_CALLS);
    assert_eq!(meta.runtimes[0].name, Some("node".to_string()));
}

#[test]
fn test_parse_runtime_def_aliases() {
    let meta = StarMetadata::parse(SAMPLE_STAR_FUNC_CALLS);
    assert_eq!(meta.runtimes[0].aliases, vec!["nodejs"]);
}

#[test]
fn test_parse_bundled_runtime_def_name() {
    let meta = StarMetadata::parse(SAMPLE_STAR_FUNC_CALLS);
    assert_eq!(meta.runtimes[1].name, Some("npm".to_string()));
    assert_eq!(meta.runtimes[2].name, Some("npx".to_string()));
}

#[test]
fn test_parse_bundled_runtime_def_bundled_with() {
    let meta = StarMetadata::parse(SAMPLE_STAR_FUNC_CALLS);
    assert_eq!(meta.runtimes[1].bundled_with, Some("node".to_string()));
    assert_eq!(meta.runtimes[2].bundled_with, Some("node".to_string()));
}

#[test]
fn test_parse_node_provider_star() {
    // Test with the actual node provider.star content
    let content = vx_provider_node_star();
    let meta = StarMetadata::parse(content);
    assert_eq!(meta.name, Some("node".to_string()));
    let names: Vec<_> = meta
        .runtimes
        .iter()
        .filter_map(|r| r.name.as_deref())
        .collect();
    assert!(
        names.contains(&"node"),
        "Expected 'node' in runtimes, got: {:?}",
        names
    );
    assert!(
        names.contains(&"npm"),
        "Expected 'npm' in runtimes, got: {:?}",
        names
    );
    assert!(
        names.contains(&"npx"),
        "Expected 'npx' in runtimes, got: {:?}",
        names
    );
}

#[test]
fn test_parse_7zip_style_aliases() {
    let meta = StarMetadata::parse(SAMPLE_STAR_7ZIP_STYLE);
    assert_eq!(
        meta.runtimes.len(),
        1,
        "Expected 1 runtime, got: {:?}",
        meta.runtimes
    );
    let rt = &meta.runtimes[0];
    assert_eq!(rt.name, Some("7zip".to_string()));
    assert_eq!(
        rt.aliases,
        vec!["7z", "7za", "7zz"],
        "Aliases not parsed correctly"
    );
}

// ---------------------------------------------------------------------------
// vx_version constraint tests
// ---------------------------------------------------------------------------

#[test]
fn test_parse_vx_version_absent() {
    let source = r#"
name = "mytool"
description = "A tool without version constraint"
runtimes = [{"name": "mytool", "executable": "mytool"}]
"#;
    let meta = StarMetadata::parse(source);
    assert_eq!(meta.vx_version, None, "Expected no vx_version constraint");
}

#[test]
fn test_parse_vx_version_gte() {
    let source = r#"
name = "newtool"
description = "A tool that requires vx >= 0.7.0"
vx_version = ">=0.7.0"
runtimes = [{"name": "newtool", "executable": "newtool"}]
"#;
    let meta = StarMetadata::parse(source);
    assert_eq!(
        meta.vx_version,
        Some(">=0.7.0".to_string()),
        "Expected vx_version = '>=0.7.0'"
    );
}

#[test]
fn test_parse_vx_version_caret() {
    let source = r#"
name = "newtool"
vx_version = "^0.8"
runtimes = [{"name": "newtool", "executable": "newtool"}]
"#;
    let meta = StarMetadata::parse(source);
    assert_eq!(
        meta.vx_version,
        Some("^0.8".to_string()),
        "Expected vx_version = '^0.8'"
    );
}

#[test]
fn test_parse_vx_version_range() {
    let source = r#"
name = "newtool"
vx_version = ">=0.7.0, <1.0.0"
runtimes = [{"name": "newtool", "executable": "newtool"}]
"#;
    let meta = StarMetadata::parse(source);
    assert_eq!(
        meta.vx_version,
        Some(">=0.7.0, <1.0.0".to_string()),
        "Expected vx_version range"
    );
}

#[test]
fn test_parse_vx_version_with_spaces() {
    let source = r#"
name        = "newtool"
vx_version  = ">=0.7.0"
runtimes = [{"name": "newtool", "executable": "newtool"}]
"#;
    let meta = StarMetadata::parse(source);
    assert_eq!(
        meta.vx_version,
        Some(">=0.7.0".to_string()),
        "Expected vx_version with extra whitespace to parse correctly"
    );
}
