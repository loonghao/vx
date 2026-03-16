//! Pure Starlark logic tests for xcodebuild provider.star
//!
//! xcodebuild is macOS-only and system-detection only (not installable by vx).

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_xcodebuild::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_xcodebuild::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_xcodebuild() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""xcodebuild""#);
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
fn test_runtimes_has_xcodebuild() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"xcodebuild" in names
"#,
    );
}

#[test]
fn test_runtimes_has_xcrun_and_swift() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"xcrun" in names and "swift" in names
"#,
    );
}

#[test]
fn test_xcodebuild_does_not_override_auto_installable() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "xcodebuild"][0]
"auto_installable" not in rt
"#,
    );
}

#[test]
fn test_xcodebuild_does_not_set_explicit_system_paths() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "xcodebuild"][0]
"system_paths" not in rt
"#,
    );
}

#[test]
fn test_xcrun_is_bundled_with_xcodebuild() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "xcrun"][0]
rt["bundled_with"] == "xcodebuild"
"#,
    );
}

#[test]
fn test_swift_is_bundled_with_xcodebuild() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "swift"][0]
rt["bundled_with"] == "xcodebuild"
"#,
    );
}

// ── download_url — always None (system-only) ──────────────────────────────────

#[test]
fn test_download_url_macos_returns_none() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "system")
url == None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_linux_returns_none() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "system")
url == None
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_sets_developer_dir() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""), install_dir = "/usr/bin", vx_home = "/home/user/.vx")
env = environment(ctx, "system")
dev_dir_ops = [op for op in env if op.get("key") == "DEVELOPER_DIR"]
len(dev_dir_ops) > 0 and "Xcode.app" in dev_dir_ops[0].get("value", "")
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
        vx_provider_xcodebuild::PROVIDER_STAR.to_string(),
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
        "store_root",
        "get_execute_path",
        "supported_platforms",
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
