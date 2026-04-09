//! Pure Starlark logic tests for yazi provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_yazi::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_yazi::PROVIDER_STAR)
}

#[test]
fn test_provider_name_is_yazi() {
    make_assert().eq(
        r#"load("provider.star", "name"); name"#,
        r#""yazi""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(
        r#"load("provider.star", "homepage"); homepage.startswith("https://")"#,
    );
}

#[test]
fn test_runtimes_has_yazi() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"yazi" in names
"#,
    );
}

#[test]
fn test_download_url_linux_x64() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-musl"))
url = download_url(ctx, "25.3.2")
url != None and ("github.com" in url or "releases" in url)
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_x64() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = "x86_64-pc-windows-msvc"))
url = download_url(ctx, "25.3.2")
url == None or ("github.com" in url or "releases" in url)
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_arm64() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = "aarch64-apple-darwin"))
url = download_url(ctx, "25.3.2")
url == None or ("github.com" in url or "releases" in url)
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_yazi::PROVIDER_STAR,
    );
}
