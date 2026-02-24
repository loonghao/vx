//! Pure Starlark logic tests for go provider.star
//!
//! These tests use `starlark::assert::Assert` to test the Starlark logic
//! directly — no network calls, no Rust runtime, just the script itself.
//!
//! The `@vx//stdlib` modules are mocked so that `load()` calls succeed
//! without needing the real VxFileLoader.

use starlark::assert::Assert;
use starlark::syntax::Dialect;

/// Build an Assert environment with mocked @vx//stdlib modules and
/// the provider.star pre-loaded as a module.
fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);

    // Mock @vx//stdlib:http.star — only the symbols used by go/provider.star
    a.module(
        "@vx//stdlib:http.star",
        r#"
def fetch_json_versions(_ctx, _url, _kind):
    """Mock: returns an empty descriptor (not used in pure-logic tests)."""
    return {"kind": _kind, "url": _url}
"#,
    );

    // Mock @vx//stdlib:install.star — only the symbols used by go/provider.star
    a.module(
        "@vx//stdlib:install.star",
        r#"
def ensure_dependencies(_runtime, check_file = None, lock_file = None, install_dir = None):
    return {"op": "ensure_dependencies", "runtime": _runtime}
"#,
    );

    a.module("provider.star", vx_provider_go::PROVIDER_STAR);
    a
}

/// Inline the provider.star content for tests that need private symbols.
fn provider_star_prefix() -> String {
    // Replace load() statements with mock implementations so Assert can evaluate
    // the script without a custom FileLoader.
    let src = vx_provider_go::PROVIDER_STAR;

    // Strip load() lines and prepend mock definitions
    let stripped: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("load("))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"
# Mock @vx//stdlib:http.star
def fetch_json_versions(_ctx, _url, _kind):
    return {{"kind": _kind, "url": _url}}

# Mock @vx//stdlib:install.star
def ensure_dependencies(_runtime, check_file = None, lock_file = None, install_dir = None):
    return {{"op": "ensure_dependencies", "runtime": _runtime}}

{}
"#,
        stripped
    )
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_go() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""go""#);
}

#[test]
fn test_provider_ecosystem_is_go() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""go""#,
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

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_go_and_gofmt() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"go" in names and "gofmt" in names
"#,
    );
}

#[test]
fn test_go_runtime_has_golang_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
go_rt = [r for r in runtimes if r["name"] == "go"][0]
"golang" in go_rt["aliases"]
"#,
    );
}

#[test]
fn test_gofmt_is_bundled_with_go() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
gofmt = [r for r in runtimes if r["name"] == "gofmt"][0]
gofmt["bundled_with"] == "go"
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_linux_x64_is_tar_gz() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
url != None and url.endswith(".tar.gz")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_linux_x64_contains_version() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
"1.22.0" in url and "linux" in url and "amd64" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_x64_is_zip() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
url != None and url.endswith(".zip")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_arm64_contains_darwin_arm64() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "1.22.0")
url != None and "darwin" in url and "arm64" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_x64_contains_darwin_amd64() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
url != None and "darwin" in url and "amd64" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_uses_go_dev() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
url.startswith("https://go.dev/dl/")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_unknown_platform_returns_none() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "freebsd", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
url == None
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_returns_archive_type() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "1.22.0")
layout["type"] == "archive"
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_strip_prefix_is_go() {
    // Go archives always have a top-level "go/" directory
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "1.22.0")
layout["strip_prefix"] == "go"
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_sets_goroot() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
env = environment(ctx, "1.22.0", "/home/user/.vx/store/go/1.22.0")
"GOROOT" in env
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_goroot_equals_install_dir() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
install_dir = "/home/user/.vx/store/go/1.22.0"
env = environment(ctx, "1.22.0", install_dir)
env["GOROOT"] == install_dir
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_path_contains_bin() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
env = environment(ctx, "1.22.0", "/opt/go")
"/bin" in env["PATH"]
"#,
        provider_star_prefix()
    ));
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    use starlark::analysis::AstModuleLint;
    use starlark::syntax::{AstModule, Dialect};
    use std::collections::HashSet;

    let ast = AstModule::parse(
        "provider.star",
        vx_provider_go::PROVIDER_STAR.to_string(),
        &Dialect::Standard,
    )
    .expect("provider.star should parse without errors");

    let known_globals: HashSet<String> = [
        "fetch_versions",
        "download_url",
        "install_layout",
        "environment",
        "pre_run",
        "ctx",
        "name",
        "description",
        "homepage",
        "repository",
        "license",
        "ecosystem",
        "runtimes",
        "permissions",
        "requires",
        "fetch_json_versions",
        "ensure_dependencies",
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
