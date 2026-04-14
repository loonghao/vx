//! Pure Starlark logic tests for uv provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_uv::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_uv::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_uv() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""uv""#);
}

#[test]
fn test_provider_ecosystem_is_python() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""python""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_uv_and_uvx() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"uv" in names and "uvx" in names
"#,
    );
}

#[test]
fn test_uvx_is_bundled_with_uv() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
uvx = [r for r in runtimes if r["name"] == "uvx"][0]
uvx["bundled_with"] == "uv"
"#,
    );
}

#[test]
fn test_uvx_reuses_uv_executable() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
uvx = [r for r in runtimes if r["name"] == "uvx"][0]
uvx["executable"] == "uv"
"#,
    );
}

#[test]
fn test_uvx_routes_through_tool_run_prefix() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
uvx = [r for r in runtimes if r["name"] == "uvx"][0]
uvx["command_prefix"] == ["tool", "run"]
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_linux_x64_is_tar_gz() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-gnu"))
url = download_url(ctx, "0.4.0")
url != None and url.endswith(".tar.gz")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_x64_is_zip() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = "x86_64-pc-windows-msvc"))
url = download_url(ctx, "0.4.0")
url != None and url.endswith(".zip")
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
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = "aarch64-apple-darwin"))
url = download_url(ctx, "0.4.0")
url != None and url.endswith(".tar.gz")
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-gnu"))
url = download_url(ctx, "0.4.0")
"github.com" in url
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-gnu"))
url = download_url(ctx, "0.4.0")
"0.4.0" in url
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_linux_is_archive() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-gnu"))
layout = install_layout(ctx, "0.4.0")
layout["type"] == "archive"
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_has_strip_prefix() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-gnu"))
layout = install_layout(ctx, "0.4.0")
len(layout["strip_prefix"]) > 0
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_prepends_install_dir() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-gnu"), install_dir = "/opt/uv", vx_home = "/home/user/.vx")
env = environment(ctx, "0.4.0")
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0 and "/opt/uv" in path_ops[0].get("value", "")
"#,
        provider_star_prefix()
    ));
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_uv::PROVIDER_STAR,
    );
}
