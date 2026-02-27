//! Pure Starlark logic tests for vcpkg provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_vcpkg::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_vcpkg::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_vcpkg() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""vcpkg""#);
}

#[test]
fn test_provider_ecosystem_is_cpp() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""cpp""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_vcpkg() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"vcpkg" in names
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_linux_returns_url() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "2024.04.26")
url != None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_returns_url() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "2024.04.26")
url != None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_returns_url() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "2024.04.26")
url != None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_uses_github() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "2024.04.26")
"github.com" in url
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_is_binary() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "2024.04.26")
layout["type"] == "binary"
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_prepends_path() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/vcpkg", vx_home = "/home/user/.vx")
env = environment(ctx, "2024.04.26")
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0
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
        vx_provider_vcpkg::PROVIDER_STAR.to_string(),
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
