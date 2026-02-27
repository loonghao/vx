//! Pure Starlark logic tests for nx provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_nx::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_nx::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_nx() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""nx""#);
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
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_nx() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"nx" in names
"#,
    );
}

#[test]
fn test_nx_runtime_has_nrwl_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "nx"][0]
"nrwl" in rt["aliases"]
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_always_returns_none() {
    // nx is an npm package alias, no direct download
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "19.0.0")
url == None
"#,
        provider_star_prefix()
    ));
}

// ── package_alias ─────────────────────────────────────────────────────────────

#[test]
fn test_package_alias_is_npm_nx() {
    make_assert().is_true(
        r#"
load("provider.star", "package_alias")
package_alias["ecosystem"] == "npm" and package_alias["package"] == "nx"
"#,
    );
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_returns_empty() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/nx", vx_home = "/home/user/.vx")
env = environment(ctx, "19.0.0")
len(env) == 0
"#,
        provider_star_prefix()
    ));
}

// ── deps logic ────────────────────────────────────────────────────────────────

#[test]
fn test_deps_nx18_requires_node18() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
d = deps(ctx, "18.0.0")
len(d) > 0 and d[0]["runtime"] == "node"
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
        vx_provider_nx::PROVIDER_STAR.to_string(),
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
        "package_alias",
        "aliases",
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
