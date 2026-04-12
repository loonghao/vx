//! Pure Starlark logic tests for rust provider.star
//!
//! Rust is managed via rustup-init binary installer.
//! After running rustup-init, rustc/cargo/rustfmt are available.

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_rust::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_rust::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_rust() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""rust""#);
}

#[test]
fn test_provider_ecosystem_is_rust() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""rust""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_rust() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"rust" in names
"#,
    );
}

#[test]
fn test_rust_runtime_has_rustup_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "rust"][0]
"rustup" in rt["aliases"]
"#,
    );
}

#[test]
fn test_runtimes_has_rustc_and_cargo() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"rustc" in names and "cargo" in names
"#,
    );
}

#[test]
fn test_rustc_is_bundled_with_rust() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "rustc"][0]
rt["bundled_with"] == "rust"
"#,
    );
}

#[test]
fn test_cargo_is_bundled_with_rust() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "cargo"][0]
rt["bundled_with"] == "rust"
"#,
    );
}

// ── package_prefixes ──────────────────────────────────────────────────────────

#[test]
fn test_package_prefixes_contains_cargo() {
    make_assert()
        .is_true(r#"load("provider.star", "package_prefixes"); "cargo" in package_prefixes"#);
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_linux_x64_returns_binary() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "1.76.0")
url != None and "rustup-init" in url
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_windows_x64_returns_exe() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "1.76.0")
url != None and url.endswith(".exe")
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_download_url_macos_arm64_returns_url() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "1.76.0")
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
url = download_url(ctx, "1.76.0")
url == None
"#,
        provider_star_prefix()
    ));
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_is_binary() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "1.76.0")
layout["type"] == "binary"
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_sets_rustup_home() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/rust/linux-x64", platform_install_dir = "/opt/rust/linux-x64", vx_home = "/home/user/.vx")
env = environment(ctx, "1.76.0")
rustup_ops = [op for op in env if op.get("key") == "RUSTUP_HOME"]
len(rustup_ops) > 0
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_sets_cargo_home() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/rust/linux-x64", platform_install_dir = "/opt/rust/linux-x64", vx_home = "/home/user/.vx")
env = environment(ctx, "1.76.0")
cargo_ops = [op for op in env if op.get("key") == "CARGO_HOME"]
len(cargo_ops) > 0
"#,
        provider_star_prefix()
    ));
}

#[test]
fn test_environment_prepends_cargo_bin_to_path() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/rust/linux-x64", platform_install_dir = "/opt/rust/linux-x64", vx_home = "/home/user/.vx")
env = environment(ctx, "1.76.0")
path_ops = [op for op in env if op.get("key") == "PATH"]
len(path_ops) > 0 and "cargo/bin" in path_ops[0].get("value", "")
"#,
        provider_star_prefix()
    ));
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_rust::PROVIDER_STAR,
    );
}
