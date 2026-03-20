//! Pure Starlark logic tests for ccache provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_ccache::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_ccache::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_ccache() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""ccache""#);
}

#[test]
fn test_provider_ecosystem_is_devtools() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""devtools""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

#[test]
fn test_provider_has_aliases() {
    make_assert().is_true(
        r#"
load("provider.star", "aliases")
"compiler-cache" in aliases
"#,
    );
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_ccache() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"ccache" in names
"#,
    );
}

#[test]
fn test_ccache_runtime_has_version_pattern() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "ccache"][0]
len(rt.get("version_pattern", "")) > 0
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_windows_x64_is_zip() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "4.9.1")
url != None and url.endswith(".zip")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_linux_x64_is_tar_xz() {
    // Linux x64 uses glibc build distributed as .tar.xz
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "4.9.1")
url != None and url.endswith(".tar.xz")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_arm64_is_tar_gz() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "4.9.1")
url != None and url.endswith(".tar.gz")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_uses_universal_binary() {
    // macOS uses a universal binary (darwin), same for x64 and arm64
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx_x64   = struct(platform = struct(os = "macos", arch = "x64",   target = ""))
ctx_arm64 = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url_x64   = download_url(ctx_x64,   "4.9.1")
url_arm64 = download_url(ctx_arm64, "4.9.1")
"darwin" in url_x64 and "darwin" in url_arm64
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
url = download_url(ctx, "4.9.1")
"4.9.1" in url
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
url = download_url(ctx, "4.9.1")
"github.com" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_linux_arm64_is_aarch64() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "arm64", target = ""))
url = download_url(ctx, "4.9.1")
url != None and "aarch64" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_unknown_platform_returns_none() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "freebsd", arch = "x64", target = ""))
url = download_url(ctx, "4.9.1")
url == None
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_windows_has_exe() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
layout = install_layout(ctx, "4.9.1")
layout["type"] == "archive" and "ccache.exe" in layout["executable_paths"]
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_linux_has_ccache() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "4.9.1")
layout["type"] == "archive" and "ccache" in layout["executable_paths"]
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_has_strip_prefix_with_version() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "4.9.1")
"4.9.1" in layout["strip_prefix"]
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_prepends_path() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/ccache", vx_home = "/home/user/.vx")
env = environment(ctx, "4.9.1")
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
        vx_provider_ccache::PROVIDER_STAR,
    );
}
