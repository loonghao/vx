//! Pure Starlark logic tests for imagemagick provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_imagemagick::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_imagemagick::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_imagemagick() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""imagemagick""#);
}

#[test]
fn test_provider_ecosystem_is_system() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""system""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_magick() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"magick" in names
"#,
    );
}

#[test]
fn test_runtimes_has_bundled_convert() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"convert" in names
"#,
    );
}

#[test]
fn test_magick_has_imagemagick_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "magick"][0]
"imagemagick" in rt["aliases"]
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_linux_x64_returns_appimage() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "7.1.1-33")
url != None and "AppImage" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_linux_arm64_returns_none() {
    // Only Linux x64 has AppImage
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "arm64", target = ""))
url = download_url(ctx, "7.1.1-33")
url == None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_returns_none() {
    // Windows uses system_install
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "7.1.1-33")
url == None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_returns_none() {
    // macOS uses system_install (brew)
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "7.1.1-33")
url == None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_linux_uses_github() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "7.1.1-33")
"github.com" in url
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_is_binary_type() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "7.1.1-33")
layout["type"] == "binary"
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_has_magick_executable() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "7.1.1-33")
"magick" in layout["executable_paths"]
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
        vx_provider_imagemagick::PROVIDER_STAR.to_string(),
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
        "deps",
        "ctx",
        "name",
        "description",
        "homepage",
        "repository",
        "license",
        "ecosystem",
        "runtimes",
        "permissions",
        "aliases",
        "system_install",
        "store_root",
        "get_execute_path",
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
