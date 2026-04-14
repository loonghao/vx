//! Pure Starlark logic tests for git provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_git::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_git::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_git() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""git""#);
}

#[test]
fn test_provider_ecosystem_is_git() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""git""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_git() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"git" in names
"#,
    );
}

#[test]
fn test_git_runtime_does_not_define_system_paths() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "git"][0]
not ("system_paths" in rt)
"#,
    );
}

#[test]
fn test_git_runtime_does_not_define_shells() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "git"][0]
not ("shells" in rt)
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_windows_x64_returns_7z_exe() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "2.44.0")
url != None and url.endswith(".7z.exe")
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
url = download_url(ctx, "2.44.0")
"github.com" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_linux_returns_none() {
    // Linux uses system package manager
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "2.44.0")
url == None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_returns_none() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "2.44.0")
url == None
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_contains_version() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "2.44.0")
"2.44.0" in url
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_windows_prepends_multiple_paths() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""), install_dir = "C:\\vx\\git", vx_home = "C:\\Users\\user\\.vx")
env = environment(ctx, "2.44.0")
len(env) > 0
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_linux_returns_empty() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/usr/bin", vx_home = "/home/user/.vx")
env = environment(ctx, "2.44.0")
len(env) == 0
"#,
        provider_star_prefix()
    ));
}

// ── windows version with .windows.N suffix ────────────────────────────────────

#[test]
fn test_download_url_windows_version_with_windows_2_suffix() {
    // Regression test: version "2.53.0.windows.2" must produce
    // tag "v2.53.0.windows.2" and asset "PortableGit-2.53.0.2-64-bit.7z.exe"
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "2.53.0.windows.2")
url != None and "v2.53.0.windows.2" in url and "PortableGit-2.53.0.2-64-bit.7z.exe" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_version_with_windows_1_suffix() {
    // version "2.53.0.windows.1" → tag "v2.53.0.windows.1", asset base "2.53.0"
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "2.53.0.windows.1")
url != None and "v2.53.0.windows.1" in url and "PortableGit-2.53.0-64-bit.7z.exe" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_arm64() {
    // arm64 arch should produce arm64 asset
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "arm64", target = ""))
url = download_url(ctx, "2.53.0.windows.2")
url != None and "arm64.7z.exe" in url
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_windows_is_archive_with_cmd_path() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
layout = install_layout(ctx, "2.44.0")
layout["type"] == "archive" and "cmd/git.exe" in layout["executable_paths"]
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_windows_includes_mingw_path() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
layout = install_layout(ctx, "2.44.0")
"mingw64/bin/git.exe" in layout["executable_paths"]
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_install_layout_linux_has_bin_git() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "2.44.0")
"bin/git" in layout["executable_paths"]
"#,
        provider_star_prefix()
    ));
}

// ── get_execute_path logic ────────────────────────────────────────────────────

#[test]
fn test_get_execute_path_windows_returns_cmd_git_exe() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""), install_dir = "C:\\vx\\git\\2.44.0")
path = get_execute_path(ctx, "2.44.0")
path.endswith("cmd/git.exe")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_get_execute_path_linux_returns_bin_git() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/home/user/.vx/store/git/2.44.0")
path = get_execute_path(ctx, "2.44.0")
path.endswith("bin/git")
"#,
        provider_star_prefix()
    ));
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_git::PROVIDER_STAR,
    );
}
