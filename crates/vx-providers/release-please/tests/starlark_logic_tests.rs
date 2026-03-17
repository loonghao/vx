//! Pure Starlark logic tests for release-please provider.star
//!
//! release-please is a Node.js tool routed via npm package alias.
//! download_url always returns None (installed via npx).

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_release_please::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_release_please::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_release_please() {
    make_assert().eq(
        r#"load("provider.star", "name"); name"#,
        r#""release-please""#,
    );
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
fn test_runtimes_has_release_please() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"release-please" in names
"#,
    );
}

// ── package_alias ─────────────────────────────────────────────────────────────

#[test]
fn test_package_alias_routes_to_npm() {
    make_assert().eq(
        r#"load("provider.star", "package_alias"); package_alias["ecosystem"]"#,
        r#""npm""#,
    );
}

#[test]
fn test_package_alias_package_is_release_please() {
    make_assert().eq(
        r#"load("provider.star", "package_alias"); package_alias["package"]"#,
        r#""release-please""#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_always_returns_none() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "16.0.0")
url == None
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_returns_empty() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/release-please", vx_home = "/home/user/.vx")
env = environment(ctx, "16.0.0")
len(env) == 0
"#,
        provider_star_prefix()
    ));
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_release_please::PROVIDER_STAR,
    );
}
