//! Pure Starlark logic tests for go provider.star
//!
//! These tests use `starlark::assert::Assert` to test the Starlark logic
//! directly — no network calls, no Rust runtime, just the script itself.
//!
//! The `@vx//stdlib` modules are mocked so that `load()` calls succeed
//! without needing the real VxFileLoader.

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

/// Build an Assert environment with mocked @vx//stdlib modules and
/// the provider.star pre-loaded as a module.
fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_go::PROVIDER_STAR);
    a
}

/// Build an Assert environment with mocks for testing provider functions directly.
fn make_assert_with_functions() -> Assert<'static> {
    use vx_starlark::test_mocks::setup_provider_test_mocks;
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    // Load provider.star as a module so its functions are available
    a.module("provider.star", vx_provider_go::PROVIDER_STAR);
    a
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_go() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""go""#);
}

#[test]
fn test_provider_ecosystem_is_go() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""go""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(
        r#"
load("provider.star", "homepage")
homepage.startswith("https://")
"#,
    );
}

#[test]
fn test_provider_has_repository() {
    make_assert().is_true(
        r#"
load("provider.star", "repository")
"github.com" in repository
"#,
    );
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_go_and_gofmt() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"go" in names and "gofmt" in names
"#,
    );
}

#[test]
fn test_go_runtime_has_golang_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
go_rt = [r for r in runtimes if r["name"] == "go"][0]
"golang" in go_rt["aliases"]
"#,
    );
}

#[test]
fn test_gofmt_is_bundled_with_go() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
gofmt = [r for r in runtimes if r["name"] == "gofmt"][0]
gofmt["bundled_with"] == "go"
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_linux_x64_is_tar_gz() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "download_url")
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
url != None and url.endswith(".tar.gz")
"#,
    );
}

#[test]
fn test_download_url_linux_x64_contains_version() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "download_url")
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
"1.22.0" in url and "linux" in url and "amd64" in url
"#,
    );
}

#[test]
fn test_download_url_windows_x64_is_zip() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "download_url")
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
url != None and url.endswith(".zip")
"#,
    );
}

#[test]
fn test_download_url_macos_arm64_contains_darwin_arm64() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "download_url")
ctx = struct(platform = struct(os = "macos", arch = "arm64", target = ""))
url = download_url(ctx, "1.22.0")
url != None and "darwin" in url and "arm64" in url
"#,
    );
}

#[test]
fn test_download_url_macos_x64_contains_darwin_amd64() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "download_url")
ctx = struct(platform = struct(os = "macos", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
url != None and "darwin" in url and "amd64" in url
"#,
    );
}

#[test]
fn test_download_url_uses_go_dev() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "download_url")
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
url.startswith("https://go.dev/dl/")
"#,
    );
}

#[test]
fn test_download_url_unknown_platform_returns_none() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "download_url")
ctx = struct(platform = struct(os = "freebsd", arch = "x64", target = ""))
url = download_url(ctx, "1.22.0")
url == None
"#,
    );
}

// ── install_layout logic ──────────────────────────────────────────────────────

#[test]
fn test_install_layout_returns_archive_type() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "install_layout")
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "1.22.0")
layout["type"] == "archive"
"#,
    );
}

#[test]
fn test_install_layout_strip_prefix_is_go() {
    // Go archives always have a top-level "go/" directory
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "install_layout")
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""))
layout = install_layout(ctx, "1.22.0")
layout["strip_prefix"] == "go"
"#,
    );
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_sets_goroot() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "environment")
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/home/user/.vx/store/go/1.22.0")
env = environment(ctx, "1.22.0")
# Check that env is a list with 2 entries (GOROOT and PATH)
type(env) == "list" and len(env) == 2
"#,
    );
}

#[test]
fn test_environment_goroot_equals_install_dir() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "environment")
install_dir = "/home/user/.vx/store/go/1.22.0"
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = install_dir)
env = environment(ctx, "1.22.0")
# Check that second entry is env_set for GOROOT with correct value (env_set returns op="set", key=...)
env[1]["op"] == "set" and env[1]["key"] == "GOROOT" and env[1]["value"] == install_dir
"#,
    );
}

#[test]
fn test_environment_path_contains_bin() {
    make_assert_with_functions().is_true(
        r#"
load("provider.star", "environment")
ctx = struct(platform = struct(os = "linux", arch = "x64", target = ""), install_dir = "/opt/go")
env = environment(ctx, "1.22.0")
# Check that first entry is prepend for PATH containing /bin
env[0]["op"] == "prepend" and env[0]["key"] == "PATH" and "/bin" in env[0]["value"]
"#,
    );
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    vx_starlark::provider_test_support::assert_provider_star_lint_clean(
        vx_provider_go::PROVIDER_STAR,
    );
}
