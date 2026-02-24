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

/// Create an Assert environment with the provider.star pre-loaded as a module.
///
/// Note: private symbols (prefixed with `_`) cannot be imported via `load()`.
/// For tests that need access to private symbols, use `make_assert_inline()`.
fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    // Use Standard dialect — same as what the engine uses for provider.star
    a.dialect(&Dialect::Standard);
    a.module("provider.star", vx_provider_python::PROVIDER_STAR);
    a
}

/// Create an Assert environment with the provider.star content inlined.
///
/// This allows tests to access private symbols like `_VERSIONS` and `_TRIPLES`
/// by embedding the provider.star source directly in the test program.
fn provider_star_prefix() -> String {
    vx_provider_python::PROVIDER_STAR.to_string()
}

// ── _VERSIONS list sanity checks ─────────────────────────────────────────────

#[test]
fn test_versions_list_is_non_empty() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
len(_VERSIONS) > 0
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_versions_list_contains_known_versions() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    // Use is_true with a single expression that checks all versions at once
    a.is_true(&format!(
        r#"
{}
versions = [v[0] for v in _VERSIONS]
(
    "3.13.4"  in versions and
    "3.12.11" in versions and
    "3.11.13" in versions and
    "3.10.18" in versions and
    "3.9.21"  in versions and
    "3.8.20"  in versions and
    "3.7.9"   in versions
)
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_versions_all_have_three_fields() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
all([len(v) == 3 for v in _VERSIONS])
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_versions_all_marked_stable() {
    // All entries in the static list are stable (no prerelease)
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
all([v[2] == False for v in _VERSIONS])
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_versions_37_use_pythonorg_date() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
py37 = [v for v in _VERSIONS if v[0].startswith("3.7.")]
all([v[1] == "pythonorg" for v in py37])
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_versions_38_plus_use_numeric_dates() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
py38_plus = [v for v in _VERSIONS if not v[0].startswith("3.7.")]
all([v[1] != "pythonorg" for v in py38_plus])
"#,
        provider_star_prefix()
    ));
}

// ── _TRIPLES platform mapping ─────────────────────────────────────────────────

#[test]
fn test_triples_contains_all_platforms() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
(
    "windows/x64"   in _TRIPLES and
    "windows/arm64" in _TRIPLES and
    "macos/x64"     in _TRIPLES and
    "macos/arm64"   in _TRIPLES and
    "linux/x64"     in _TRIPLES and
    "linux/arm64"   in _TRIPLES
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
(
    _TRIPLES["windows/x64"]   == "x86_64-pc-windows-msvc" and
    _TRIPLES["windows/arm64"] == "aarch64-pc-windows-msvc" and
    _TRIPLES["macos/x64"]     == "x86_64-apple-darwin" and
    _TRIPLES["macos/arm64"]   == "aarch64-apple-darwin" and
    _TRIPLES["linux/x64"]     == "x86_64-unknown-linux-gnu" and
    _TRIPLES["linux/arm64"]   == "aarch64-unknown-linux-gnu"
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
_TRIPLES.get("unknown/arch") == None
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

#[test]
fn test_python_runtime_has_priority() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
python = [r for r in runtimes if r["name"] == "python"][0]
python["priority"] == 100
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

// ── fetch_versions logic (pure Starlark, no network) ─────────────────────────

#[test]
fn test_fetch_versions_returns_list_of_dicts() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# Simulate a minimal ctx struct
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
result = fetch_versions(ctx)
type(result) == "list" and len(result) > 0
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_fetch_versions_each_entry_has_version_and_stable() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
result = fetch_versions(ctx)
all(["version" in v and "stable" in v for v in result])
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
result = fetch_versions(ctx)
all([v["stable"] == True for v in result])
"#,
        provider_star_prefix()
    ));
}

// ── download_url logic (pure Starlark, no network) ───────────────────────────

#[test]
fn test_download_url_linux_x64_contains_version_and_date() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
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
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
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
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "9.99.0")
url == None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_python37_linux_returns_none() {
    // Python 3.7 is Windows-only
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "3.7.9")
url == None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_python37_windows_returns_zip() {
    // Python 3.7 on Windows uses Python.org embeddable (.zip)
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "3.7.9")
url != None and url.endswith(".zip") and "python.org" in url
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
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
