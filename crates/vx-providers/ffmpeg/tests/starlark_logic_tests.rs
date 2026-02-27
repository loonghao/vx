//! Pure Starlark logic tests for ffmpeg provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_ffmpeg::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_ffmpeg::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_ffmpeg() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""ffmpeg""#);
}

#[test]
fn test_provider_ecosystem_is_media() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""media""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_ffmpeg() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"ffmpeg" in names
"#,
    );
}

#[test]
fn test_ffmpeg_runtime_has_avconv_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "ffmpeg"][0]
"avconv" in rt["aliases"]
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_windows_is_zip() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "7.0")
url != None and url.endswith(".zip")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_uses_github() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "7.0")
"github.com" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_linux_x64_uses_johnvansickle() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "7.0")
url != None and "johnvansickle.com" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_linux_arm64_uses_johnvansickle() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "arm64", target = ""))
url = download_url(ctx, "7.0")
url != None and "johnvansickle.com" in url and "arm64" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_uses_evermeet() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "x64", target = ""))
url = download_url(ctx, "7.0")
url != None and "evermeet.cx" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_unknown_arch_returns_none() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "riscv64", target = ""))
url = download_url(ctx, "7.0")
url == None
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_windows_has_bin_prefix() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
layout = install_layout(ctx, "7.0")
any(["bin/" in p for p in layout["executable_paths"]])
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_linux_has_ffmpeg_executable() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "7.0")
"ffmpeg" in layout["executable_paths"]
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_windows_prepends_bin_dir() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""), install_dir = "C:\\vx\\ffmpeg", vx_home = "C:\\Users\\user\\.vx")
env = environment(ctx, "7.0")
path_ops = [op for op in env if op.get("name") == "PATH"]
len(path_ops) > 0 and "bin" in path_ops[0].get("value", "")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_linux_prepends_install_dir() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/ffmpeg", vx_home = "/home/user/.vx")
env = environment(ctx, "7.0")
path_ops = [op for op in env if op.get("name") == "PATH"]
len(path_ops) > 0 and "/opt/ffmpeg" in path_ops[0].get("value", "")
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
        vx_provider_ffmpeg::PROVIDER_STAR.to_string(),
        &Dialect::Standard,
    )
    .expect("provider.star should parse without errors");

    let known_globals: HashSet<String> = [
        "fetch_versions",
        "download_url",
        "install_layout",
        "environment",
        "post_install",
        "post_extract",
        "pre_run",
        "uninstall",
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
        "aliases",
        "store_root",
        "get_execute_path",
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
