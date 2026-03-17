//! Pure Starlark logic tests for choco provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_choco::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_choco::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_choco() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""choco""#);
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
fn test_runtimes_has_choco() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"choco" in names
"#,
    );
}

#[test]
fn test_choco_runtime_has_chocolatey_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "choco"][0]
"chocolatey" in rt["aliases"]
"#,
    );
}

#[test]
fn test_choco_runtime_does_not_define_system_paths() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "choco"][0]
not ("system_paths" in rt)
"#,
    );
}

// ── platform constraint ───────────────────────────────────────────────────────

#[test]
fn test_supported_platforms_windows_only() {
    make_assert().is_true(
        r#"
load("provider.star", "supported_platforms")
platforms = supported_platforms()
all([p["os"] == "windows" for p in platforms])
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_returns_none() {
    // choco is installed via PowerShell script, not direct download
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "2.3.0")
url == None
"#,
        provider_star_prefix()
    ));
}

// ── script_install ────────────────────────────────────────────────────────────

#[test]
fn test_script_install_is_defined() {
    make_assert().is_true(
        r#"
load("provider.star", "script_install")
script_install != None
"#,
    );
}

#[test]
fn test_script_install_has_url() {
    make_assert().is_true(
        r#"
load("provider.star", "script_install")
"chocolatey.org" in script_install.get("url", "")
"#,
    );
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_sets_chocolatey_install() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""), install_dir = "C:/vx/store/choco/2.3.0", vx_home = "C:/Users/user/.vx")
env = environment(ctx, "2.3.0")
choco_ops = [op for op in env if op.get("key") == "ChocolateyInstall"]
len(choco_ops) > 0
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_prepends_bin_to_path() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""), install_dir = "C:/vx/store/choco/2.3.0", vx_home = "C:/Users/user/.vx")
env = environment(ctx, "2.3.0")
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0
"#,
        provider_star_prefix()
    ));
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_choco::PROVIDER_STAR,
    );
}
