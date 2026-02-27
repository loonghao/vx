//! Pure Starlark logic tests for brew provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_brew::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_brew::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_brew() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""brew""#);
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
fn test_runtimes_has_brew() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"brew" in names
"#,
    );
}

#[test]
fn test_brew_runtime_has_homebrew_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "brew"][0]
"homebrew" in rt["aliases"]
"#,
    );
}

#[test]
fn test_brew_runtime_has_system_paths() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "brew"][0]
len(rt["system_paths"]) > 0
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_always_returns_none() {
    // brew is installed via shell script, not direct download
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "4.0.0")
url == None
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_macos_arm64_prepends_homebrew_path() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""), install_dir = "/opt/homebrew", vx_home = "/home/user/.vx")
env = environment(ctx, "4.0.0")
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0 and "/opt/homebrew/bin" in path_ops[0].get("value", "")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_macos_x64_prepends_usr_local() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "x64", target = ""), install_dir = "/usr/local", vx_home = "/home/user/.vx")
env = environment(ctx, "4.0.0")
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0 and "/usr/local/bin" in path_ops[0].get("value", "")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_linux_prepends_linuxbrew() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/home/linuxbrew/.linuxbrew", vx_home = "/home/user/.vx")
env = environment(ctx, "4.0.0")
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0 and "linuxbrew" in path_ops[0].get("value", "")
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
        vx_provider_brew::PROVIDER_STAR.to_string(),
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
        "script_install",
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
