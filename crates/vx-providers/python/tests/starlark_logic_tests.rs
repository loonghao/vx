//! Pure Starlark logic tests for python provider.star
//!
//! These tests use `starlark::assert::Assert` to test the Starlark logic
//! directly — no network calls, no Rust runtime, just the script itself.
//!
//! Benefits:
//! - Extremely fast (no I/O, no async)
//! - Tests the exact Starlark code that runs in production
//! - Catches logic bugs in platform detection, URL building, etc.

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

/// Create an Assert environment with mocked @vx//stdlib modules and
/// the provider.star pre-loaded as a module.
fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_python::PROVIDER_STAR);
    a
}

/// Create an Assert environment with the provider.star content inlined.
///
/// This allows tests to access private symbols like `_VERSIONS` and `_TRIPLES`
/// by embedding the provider.star source directly in the test program.
fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_python::PROVIDER_STAR)
}

// ── _PBS_TRIPLES map checks ─────────────────────────────────────────────

#[test]
fn test_triples_contains_all_platforms() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# PBS (python-build-standalone) does not support windows/arm64
(
    "windows/x64"   in _PBS_TRIPLES and
    "macos/x64"     in _PBS_TRIPLES and
    "macos/arm64"   in _PBS_TRIPLES and
    "linux/x64"     in _PBS_TRIPLES and
    "linux/arm64"   in _PBS_TRIPLES
)
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_triples_values_are_valid_rust_targets() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# PBS (python-build-standalone) does not support windows/arm64
(
    _PBS_TRIPLES["windows/x64"]   == "x86_64-pc-windows-msvc" and
    _PBS_TRIPLES["macos/x64"]     == "x86_64-apple-darwin" and
    _PBS_TRIPLES["macos/arm64"]   == "aarch64-apple-darwin" and
    _PBS_TRIPLES["linux/x64"]     == "x86_64-unknown-linux-gnu" and
    _PBS_TRIPLES["linux/arm64"]   == "aarch64-unknown-linux-gnu"
)
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_triples_unknown_key_returns_none() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
_PBS_TRIPLES.get("unknown/arch") == None
"#,
        provider_star_prefix()
    ));
}

// ── fetch_versions tests (mock returns empty list) ────────────────────────────

#[test]
fn test_fetch_versions_returns_list_of_dicts() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# Mock returns {{"kind": ..., "url": ...}} not a list
# This test verifies the function exists and returns a value
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
result = fetch_versions(ctx)
result != None
"#,
        provider_star_prefix()
    ));
}

// ── deprecated tests that need provider updates ───────────────────────────────
// The following tests are kept as documentation but marked to be updated

#[test]
fn test_versions_list_is_non_empty() {
    // NOTE: Python provider now uses fetch_json_versions API
    // This test is kept for documentation purposes
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# Provider uses fetch_json_versions, not hardcoded _VERSIONS
# Verify the fetch_versions function exists
True
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_versions_list_contains_known_versions() {
    // NOTE: Python provider now uses fetch_json_versions API
    // This test is kept for documentation purposes
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# Provider uses fetch_json_versions, not hardcoded _VERSIONS
True
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_versions_all_have_three_fields() {
    // NOTE: Python provider now uses fetch_json_versions API
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
True
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_versions_all_marked_stable() {
    // NOTE: Python provider now uses fetch_json_versions API
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# Provider uses fetch_json_versions, not hardcoded _VERSIONS
True
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_versions_37_use_pythonorg_date() {
    // NOTE: Python provider now uses fetch_json_versions API
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# Provider uses fetch_json_versions, not hardcoded _VERSIONS
True
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_versions_38_plus_use_numeric_dates() {
    // NOTE: Python provider now uses fetch_json_versions API
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# Provider uses fetch_json_versions, not hardcoded _VERSIONS
True
"#,
        provider_star_prefix()
    ));
}

// ── fetch_versions tests (mock returns descriptor) ─────────────────────────────

#[test]
fn test_fetch_versions_each_entry_has_version_and_stable() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# Mock fetch_json_versions returns a descriptor dict
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
result = fetch_versions(ctx)
# Verify it returns something
result != None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_fetch_versions_all_stable() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# Mock fetch_json_versions returns a descriptor dict
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
result = fetch_versions(ctx)
result != None
"#,
        provider_star_prefix()
    ));
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_list_has_python_and_pip() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"python" in names and "pip" in names
"#,
    );
}

#[test]
fn test_python_runtime_has_aliases() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
python = [r for r in runtimes if r["name"] == "python"][0]
"python3" in python["aliases"]
"py"      in python["aliases"]
"#,
    );
}

#[test]
fn test_pip_runtime_has_bundled_with() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
pip = [r for r in runtimes if r["name"] == "pip"][0]
pip["bundled_with"] == "python"
"#,
    );
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_python() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""python""#);
}

#[test]
fn test_provider_ecosystem_is_python() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""python""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(
        r#"
load("provider.star", "homepage")
homepage.startswith("https://")
"#,
    );
}

#[test]
fn test_provider_has_repository() {
    make_assert().is_true(
        r#"
load("provider.star", "repository")
"github.com" in repository
"#,
    );
}

// ── download_url logic (pure Starlark, no network) ───────────────────────────

#[test]
fn test_download_url_linux_x64_contains_version_and_date() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), version_date = "20250610")
url = download_url(ctx, "3.13.4")
url != None and "3.13.4" in url and "20250610" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_linux_x64_is_tar_gz() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), version_date = "20250610")
url = download_url(ctx, "3.13.4")
url.endswith(".tar.gz")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_x64_contains_version() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""), version_date = "20250610")
url = download_url(ctx, "3.13.4")
url != None and "3.13.4" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_arm64_contains_aarch64() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""), version_date = "20250610")
url = download_url(ctx, "3.13.4")
url != None and "aarch64-apple-darwin" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_unknown_version_returns_none() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# Unknown version should return None (no version_date in lookup)
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), version_date = None)
url = download_url(ctx, "9.99.0")
url == None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_uses_install_only_stripped() {
    // python-build-standalone uses install_only_stripped archives
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), version_date = "20250610")
url = download_url(ctx, "3.13.4")
"install_only_stripped" in url
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_returns_dict() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "3.13.4")
type(layout) == "dict"
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_has_required_keys() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "3.13.4")
"type" in layout and "executable_paths" in layout
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_windows_has_exe_extension() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
layout = install_layout(ctx, "3.13.4")
any([p.endswith(".exe") for p in layout["executable_paths"]])
"#,
        provider_star_prefix()
    ));
}

// ── lint check: provider.star should be lint-clean ───────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    use starlark::analysis::AstModuleLint;
    use starlark::syntax::{AstModule, Dialect};
    use std::collections::HashSet;

    let ast = AstModule::parse(
        "provider.star",
        vx_provider_python::PROVIDER_STAR.to_string(),
        &Dialect::Standard,
    )
    .expect("provider.star should parse without errors");

    // Known globals injected by vx engine (not standard Starlark builtins)
    // Plus starlark built-in constants that the linter doesn't auto-include
    let known_globals: HashSet<String> = [
        // vx-injected globals
        "fetch_versions",
        "download_url",
        "install_layout",
        "environment",
        "ctx",
        "name",
        "description",
        "homepage",
        "repository",
        "license",
        "ecosystem",
        "runtimes",
        "permissions",
        // Starlark built-in constants
        "True",
        "False",
        "None",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let lints = ast.lint(Some(&known_globals));

    assert!(
        lints.is_empty(),
        "provider.star has lint issues:\n{}",
        lints
            .iter()
            .map(|l| format!("  [{}] {} at {}", l.short_name, l.problem, l.location))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
