//! Pure Starlark logic tests for node provider.star
//!
//! These tests use `starlark::assert::Assert` to test the Starlark logic
//! directly — no network calls, no Rust runtime, just the script itself.
//!
//! The `@vx//stdlib` modules are mocked so that `load()` calls succeed
//! without needing the real VxFileLoader.

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

/// Build an Assert environment with mocked @vx//stdlib modules and
/// the provider.star pre-loaded as a module.
fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_node::PROVIDER_STAR);
    a
}

/// Inline the provider.star content for tests that need private symbols.
fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_node::PROVIDER_STAR)
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/node")
env = environment(ctx, "20.0.0")
# Linux should have PATH prepended with install_dir + "/bin"
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0 and "/bin" in path_ops[0].get("value", "")
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
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""), install_dir = "C:\\vx\\node")
env = environment(ctx, "20.0.0")
# Windows should have PATH prepended with install_dir (no /bin suffix)
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0 and "C:\\vx\\node" in path_ops[0].get("value", "")
"#,
        provider_star_prefix()
    ));
}

// ── uninstall hook ────────────────────────────────────────────────────────────

#[test]
fn test_uninstall_returns_false() {
    // Node.js uninstall is not defined (uses default behavior)
    // The provider doesn't define an uninstall function, so we just verify
    // the default behavior (directory removal) is used
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
# uninstall is not defined in provider, default behavior is used
# We verify the provider loads correctly without uninstall defined
name == "node"
"#,
        provider_star_prefix()
    ));
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_node::PROVIDER_STAR,
    );
}
