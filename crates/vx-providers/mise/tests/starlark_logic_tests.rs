//! Pure Starlark logic tests for mise provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_mise::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_mise::PROVIDER_STAR)
}

#[test]
fn test_provider_name_is_mise() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""mise""#);
}

#[test]
fn test_provider_ecosystem_is_devtools() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""devtools""#,
    );
}

#[test]
fn test_download_url_linux_x64_is_tar_gz() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-musl"))
url = download_url(ctx, "2026.3.18")
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
url = download_url(ctx, "2026.3.18")
url != None and url.endswith(".zip")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_linux_uses_top_level_mise_prefix() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-musl"))
layout = install_layout(ctx, "2026.3.18")
layout["type"] == "archive" and layout["strip_prefix"] == "mise"
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_linux_includes_bin_path() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-musl"))
layout = install_layout(ctx, "2026.3.18")
"bin/mise" in layout["executable_paths"]
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_windows_uses_strip_prefix_mise_bin() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = "x86_64-pc-windows-msvc"))
layout = install_layout(ctx, "2026.3.18")
layout["strip_prefix"] == "mise/bin" and "mise.exe" in layout["executable_paths"]
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_prepends_bin_dir() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = "x86_64-unknown-linux-musl"), install_dir = "/opt/mise", vx_home = "/home/user/.vx")
env = environment(ctx, "2026.3.18")
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0 and path_ops[0].get("value") == "/opt/mise/bin"
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_mise::PROVIDER_STAR,
    );
}
