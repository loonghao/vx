//! Pure Starlark logic tests for buf provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_buf::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_buf::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_buf() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""buf""#);
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_buf() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"buf" in names
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_linux_x64() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/buf/1.50.0")
url = download_url(ctx, "1.50.0")
url != None and "Linux" in url and "x86_64" in url and url.endswith(".tar.gz")
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
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/buf/1.50.0")
url = download_url(ctx, "1.50.0")
url != None and "Windows" in url and "x86_64" in url and url.endswith(".zip")
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
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/buf/1.50.0")
url = download_url(ctx, "1.50.0")
url != None and "Darwin" in url and "arm64" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_x64() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "x64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/buf/1.50.0")
url = download_url(ctx, "1.50.0")
url != None and "Darwin" in url and "x86_64" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_unsupported_platform_returns_none() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "freebsd", arch = "x64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/buf/1.50.0")
url = download_url(ctx, "1.50.0")
url == None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_contains_github_host() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/buf/1.50.0")
url = download_url(ctx, "1.50.0")
"github.com" in url or "githubusercontent.com" in url
"#,
        provider_star_prefix()
    ));
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_buf::PROVIDER_STAR,
    );
}
