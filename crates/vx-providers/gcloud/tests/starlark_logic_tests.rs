//! Pure Starlark logic tests for gcloud provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_gcloud::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_gcloud::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_gcloud() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""gcloud""#);
}

#[test]
fn test_provider_ecosystem_is_cloud() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""cloud""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_gcloud() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"gcloud" in names
"#,
    );
}

#[test]
fn test_runtimes_has_bundled_gsutil_and_bq() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"gsutil" in names and "bq" in names
"#,
    );
}

#[test]
fn test_gcloud_has_google_cloud_sdk_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "gcloud"][0]
"google-cloud-sdk" in rt["aliases"]
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
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "470.0.0")
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
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "470.0.0")
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
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "470.0.0")
url != None and url.endswith(".tar.gz")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_uses_dl_google_com() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "470.0.0")
"dl.google.com" in url
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
url = download_url(ctx, "470.0.0")
"470.0.0" in url
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_is_archive_with_strip_prefix() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "470.0.0")
layout["__type"] == "archive" and layout["strip_prefix"] == "google-cloud-sdk"
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_linux_has_gcloud_executable() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "470.0.0")
"bin/gcloud" in layout["executable_paths"]
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_windows_has_cmd_executables() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
layout = install_layout(ctx, "470.0.0")
"bin/gcloud.cmd" in layout["executable_paths"]
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_prepends_bin_dir() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/gcloud", vx_home = "/home/user/.vx")
env = environment(ctx, "470.0.0")
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0 and "/opt/gcloud/bin" in path_ops[0].get("value", "")
"#,
        provider_star_prefix()
    ));
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_gcloud::PROVIDER_STAR,
    );
}
