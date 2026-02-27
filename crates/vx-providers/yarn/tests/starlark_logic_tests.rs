//! Pure Starlark logic tests for yarn provider.star
//!
//! yarn is a cross-platform Node.js package manager.
//! Asset: yarn-v{version}.tar.gz (same for all platforms).

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_yarn::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_yarn::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_yarn() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""yarn""#);
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

#[test]
fn test_provider_has_package_prefixes() {
    make_assert()
        .is_true(r#"load("provider.star", "package_prefixes"); "yarn" in package_prefixes"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_yarn() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"yarn" in names
"#,
    );
}

#[test]
fn test_yarn_runtime_has_yarnpkg_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "yarn"][0]
"yarnpkg" in rt["aliases"]
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_linux_is_tar_gz() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "1.22.22")
url != None and url.endswith(".tar.gz")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_is_tar_gz() {
    // yarn uses same tar.gz for all platforms
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "1.22.22")
url != None and url.endswith(".tar.gz")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_is_tar_gz() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "1.22.22")
url != None and url.endswith(".tar.gz")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_contains_version() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "1.22.22")
"1.22.22" in url
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
url = download_url(ctx, "1.22.22")
"github.com" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_uses_yarn_v_prefix() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "1.22.22")
"yarn-v1.22.22" in url
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_is_archive() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "1.22.22")
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
layout = install_layout(ctx, "1.22.22")
"1.22.22" in layout["strip_prefix"]
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_has_yarn_executable() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "1.22.22")
any(["yarn" in p for p in layout["executable_paths"]])
"#,
        provider_star_prefix()
    ));
}

// ── deps — version-based Node.js requirement ──────────────────────────────────

#[test]
fn test_deps_yarn_v1_requires_node_12() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
d = deps(ctx, "1.22.22")
len(d) > 0 and d[0]["runtime"] == "node"
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_deps_yarn_v4_requires_node_18() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
d = deps(ctx, "4.0.0")
len(d) > 0 and "18" in d[0]["version"]
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_prepends_bin_path() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/yarn", vx_home = "/home/user/.vx")
env = environment(ctx, "1.22.22")
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0 and "/opt/yarn/bin" in path_ops[0].get("value", "")
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
        vx_provider_yarn::PROVIDER_STAR.to_string(),
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
        "package_prefixes",
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
