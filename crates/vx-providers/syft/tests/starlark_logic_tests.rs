//! Pure Starlark logic tests for syft provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_syft::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_syft::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_syft() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""syft""#);
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_syft() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"syft" in names
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/syft/1.42.4")
url = download_url(ctx, "1.42.4")
url != None and "linux" in url and "amd64" in url and url.endswith(".tar.gz")
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
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/syft/1.42.4")
url = download_url(ctx, "1.42.4")
url != None and "windows" in url and "amd64" in url
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
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/syft/1.42.4")
url = download_url(ctx, "1.42.4")
url != None and "darwin" in url and "arm64" in url
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/syft/1.42.4")
url = download_url(ctx, "1.42.4")
"1.42.4" in url
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), vx_home = "/tmp/vx", install_dir = "/tmp/vx/store/syft/1.42.4")
url = download_url(ctx, "1.42.4")
"github.com" in url or "githubusercontent.com" in url
"#,
        provider_star_prefix()
    ));
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_syft::PROVIDER_STAR,
    );
}
