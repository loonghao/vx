//! Pure Starlark logic tests for node provider.star
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

    // Mock @vx//stdlib:http.star
    a.module(
        "@vx//stdlib:http.star",
        r#"
def fetch_json_versions(_ctx, _url, _kind):
    return {"kind": _kind, "url": _url}
"#,
    );

    // Mock @vx//stdlib:install.star
    a.module(
        "@vx//stdlib:install.star",
        r#"
def set_permissions(_path, _mode):
    return {"op": "set_permissions", "path": _path, "mode": _mode}

def ensure_dependencies(_runtime, check_file = None, lock_file = None, install_dir = None):
    return {"op": "ensure_dependencies", "runtime": _runtime}
"#,
    );

    a.module("provider.star", vx_provider_node::PROVIDER_STAR);
    a
}

/// Inline the provider.star content for tests that need private symbols.
fn provider_star_prefix() -> String {
    let src = vx_provider_node::PROVIDER_STAR;

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
def set_permissions(_path, _mode):
    return {{"op": "set_permissions", "path": _path, "mode": _mode}}

def ensure_dependencies(_runtime, check_file = None, lock_file = None, install_dir = None):
    return {{"op": "ensure_dependencies", "runtime": _runtime}}

{}
"#,
        stripped
    )
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_node() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""node""#);
}

#[test]
fn test_provider_ecosystem_is_nodejs() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""nodejs""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(
        r#"
load("provider.star", "homepage")
homepage.startswith("https://nodejs.org")
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
fn test_runtimes_has_node_npm_npx() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"node" in names and "npm" in names and "npx" in names
"#,
    );
}

#[test]
fn test_node_runtime_has_nodejs_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
node_rt = [r for r in runtimes if r["name"] == "node"][0]
"nodejs" in node_rt["aliases"]
"#,
    );
}

#[test]
fn test_npm_is_bundled_with_node() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
npm = [r for r in runtimes if r["name"] == "npm"][0]
npm["bundled_with"] == "node"
"#,
    );
}

#[test]
fn test_npx_is_bundled_with_node() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
npx = [r for r in runtimes if r["name"] == "npx"][0]
npx["bundled_with"] == "node"
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_linux_x64_is_tar_xz() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "20.0.0")
url != None and url.endswith(".tar.xz")
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
url = download_url(ctx, "20.0.0")
"20.0.0" in url and "linux" in url and "x64" in url
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
url = download_url(ctx, "20.0.0")
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
url = download_url(ctx, "20.0.0")
url != None and "darwin" in url and "arm64" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_uses_nodejs_org() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "20.0.0")
url.startswith("https://nodejs.org/dist/")
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
url = download_url(ctx, "20.0.0")
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
layout = install_layout(ctx, "20.0.0")
layout["type"] == "archive"
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_strip_prefix_contains_version() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "20.0.0")
"20.0.0" in layout["strip_prefix"]
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_windows_strip_prefix_contains_win() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
layout = install_layout(ctx, "20.0.0")
"win" in layout["strip_prefix"]
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_linux_path_contains_bin() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
env = environment(ctx, "20.0.0", "/opt/node")
"/bin" in env["PATH"]
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_windows_path_is_install_dir() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
env = environment(ctx, "20.0.0", "C:\\vx\\node")
env["PATH"] == "C:\\vx\\node"
"#,
        provider_star_prefix()
    ));
}

// ── uninstall hook ────────────────────────────────────────────────────────────

#[test]
fn test_uninstall_returns_false() {
    // Node.js uninstall delegates to default directory removal
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
result = uninstall(ctx, "20.0.0")
result == False
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
        vx_provider_node::PROVIDER_STAR.to_string(),
        &Dialect::Standard,
    )
    .expect("provider.star should parse without errors");

    let known_globals: HashSet<String> = [
        "fetch_versions",
        "download_url",
        "install_layout",
        "environment",
        "post_install",
        "pre_run",
        "uninstall",
        "ctx",
        "name",
        "description",
        "homepage",
        "repository",
        "license",
        "ecosystem",
        "runtimes",
        "permissions",
        "fetch_json_versions",
        "set_permissions",
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
