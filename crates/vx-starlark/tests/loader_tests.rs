//! Tests for the VxModuleLoader (@vx//stdlib module system)

use vx_starlark::VxModuleLoader;

// ============================================================
// Module detection
// ============================================================

#[test]
fn test_is_vx_module_stdlib() {
    assert!(VxModuleLoader::is_vx_module("@vx//stdlib:semver.star"));
    assert!(VxModuleLoader::is_vx_module("@vx//stdlib:platform.star"));
    assert!(VxModuleLoader::is_vx_module("@vx//stdlib:http.star"));
    assert!(VxModuleLoader::is_vx_module("@vx//stdlib:provider.star"));
    assert!(VxModuleLoader::is_vx_module("@vx//stdlib:runtime.star"));
    assert!(VxModuleLoader::is_vx_module("@vx//stdlib:layout.star"));
}

#[test]
fn test_is_vx_module_non_vx() {
    assert!(!VxModuleLoader::is_vx_module("./local.star"));
    assert!(!VxModuleLoader::is_vx_module("../shared.star"));
    assert!(!VxModuleLoader::is_vx_module("@bazel//rules:foo.bzl"));
    assert!(!VxModuleLoader::is_vx_module("provider.star"));
}

// ============================================================
// Module source retrieval
// ============================================================

#[test]
fn test_get_source_semver() {
    let loader = VxModuleLoader::new();
    let source = loader.get_source("@vx//stdlib:semver.star");
    assert!(source.is_some(), "semver.star should be available");
    let src = source.unwrap();
    // Should contain key functions
    assert!(
        src.contains("semver_compare"),
        "Should contain semver_compare"
    );
    assert!(
        src.contains("semver_strip_v"),
        "Should contain semver_strip_v"
    );
    assert!(src.contains("semver_sort"), "Should contain semver_sort");
}

#[test]
fn test_get_source_platform() {
    let loader = VxModuleLoader::new();
    let source = loader.get_source("@vx//stdlib:platform.star");
    assert!(source.is_some(), "platform.star should be available");
    let src = source.unwrap();
    assert!(src.contains("is_windows"), "Should contain is_windows");
    assert!(
        src.contains("platform_triple"),
        "Should contain platform_triple"
    );
    assert!(src.contains("arch_to_gnu"), "Should contain arch_to_gnu");
}

#[test]
fn test_get_source_http() {
    let loader = VxModuleLoader::new();
    let source = loader.get_source("@vx//stdlib:http.star");
    assert!(source.is_some(), "http.star should be available");
    let src = source.unwrap();
    assert!(
        src.contains("github_releases"),
        "Should contain github_releases"
    );
    assert!(
        src.contains("github_download_url"),
        "Should contain github_download_url"
    );
    assert!(
        src.contains("parse_github_tag"),
        "Should contain parse_github_tag"
    );
}

#[test]
fn test_get_source_unknown_module() {
    let loader = VxModuleLoader::new();
    let source = loader.get_source("@vx//stdlib:nonexistent.star");
    assert!(source.is_none(), "Unknown module should return None");
}

// ============================================================
// Available modules list
// ============================================================

#[test]
fn test_available_modules_contains_all_builtins() {
    let loader = VxModuleLoader::new();
    let modules = loader.available_modules();

    assert!(
        modules.contains(&"@vx//stdlib:semver.star"),
        "Should list semver.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:platform.star"),
        "Should list platform.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:http.star"),
        "Should list http.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:github.star"),
        "Should list github.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:install.star"),
        "Should list install.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:env.star"),
        "Should list env.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:layout.star"),
        "Should list layout.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:permissions.star"),
        "Should list permissions.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:provider.star"),
        "Should list provider.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:provider_templates.star"),
        "Should list provider_templates.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:runtime.star"),
        "Should list runtime.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:script_install.star"),
        "Should list script_install.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:system_install.star"),
        "Should list system_install.star"
    );
    assert!(
        modules.contains(&"@vx//stdlib:test.star"),
        "Should list test.star"
    );
}

#[test]
fn test_available_modules_count() {
    let loader = VxModuleLoader::new();
    let modules = loader.available_modules();
    // We have 14 built-in modules:
    // semver, platform, http, github, install, env,
    // layout, permissions, provider, provider_templates,
    // runtime, script_install, system_install, test
    assert_eq!(modules.len(), 14, "Should have exactly 14 built-in modules");
}

// ============================================================
// Default implementation
// ============================================================

#[test]
fn test_default_loader_same_as_new() {
    let loader1 = VxModuleLoader::new();
    let loader2 = VxModuleLoader::default();

    // Both should have the same modules
    let mut m1 = loader1.available_modules();
    let mut m2 = loader2.available_modules();
    m1.sort();
    m2.sort();
    assert_eq!(m1, m2);
}

// ============================================================
// Starlark content validity
// ============================================================

#[test]
fn test_semver_star_is_valid_starlark() {
    let loader = VxModuleLoader::new();
    let source = loader.get_source("@vx//stdlib:semver.star").unwrap();

    // Basic syntax checks
    assert!(!source.is_empty(), "Source should not be empty");
    // Should not contain Python-specific import syntax (at line start)
    assert!(
        !source.contains("\nimport "),
        "Should not use Python import"
    );
    assert!(
        !source.contains("\nfrom "),
        "Should not use Python from-import"
    );
    // Should use def for functions
    assert!(source.contains("def "), "Should define functions with def");
}

#[test]
fn test_platform_star_is_valid_starlark() {
    let loader = VxModuleLoader::new();
    let source = loader.get_source("@vx//stdlib:platform.star").unwrap();

    assert!(!source.is_empty());
    assert!(
        !source.contains("\nimport "),
        "Should not use Python import"
    );
    assert!(source.contains("def "), "Should define functions with def");
}

#[test]
fn test_http_star_is_valid_starlark() {
    let loader = VxModuleLoader::new();
    let source = loader.get_source("@vx//stdlib:http.star").unwrap();

    assert!(!source.is_empty());
    assert!(
        !source.contains("\nimport "),
        "Should not use Python import"
    );
    assert!(source.contains("def "), "Should define functions with def");
}
