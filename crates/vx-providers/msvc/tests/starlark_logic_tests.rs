//! Pure Starlark logic tests for msvc provider.star

use starlark::assert::Assert;
use starlark::syntax::Dialect;
use vx_starlark::test_mocks::setup_provider_test_mocks;

fn make_assert() -> Assert<'static> {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a.module("provider.star", vx_provider_msvc::PROVIDER_STAR);
    a
}

fn provider_star_prefix() -> String {
    use vx_starlark::test_mocks::prepare_provider_source;
    prepare_provider_source(vx_provider_msvc::PROVIDER_STAR)
}

// ── provider metadata ─────────────────────────────────────────────────────────

#[test]
fn test_provider_name_is_msvc() {
    make_assert().eq(r#"load("provider.star", "name"); name"#, r#""msvc""#);
}

#[test]
fn test_provider_ecosystem_is_system() {
    make_assert().eq(
        r#"load("provider.star", "ecosystem"); ecosystem"#,
        r#""system""#,
    );
}

#[test]
fn test_provider_has_homepage() {
    make_assert().is_true(r#"load("provider.star", "homepage"); homepage.startswith("https://")"#);
}

// ── runtimes metadata ─────────────────────────────────────────────────────────

#[test]
fn test_runtimes_has_msvc() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"msvc" in names
"#,
    );
}

#[test]
fn test_runtimes_has_bundled_tools() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"nmake" in names and "link" in names and "ml64" in names
"#,
    );
}

#[test]
fn test_runtimes_has_windows_sdk_tools() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"mt" in names and "rc" in names and "signtool" in names
"#,
    );
}

#[test]
fn test_runtimes_has_managed_tools() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
names = [r["name"] for r in runtimes]
"csc" in names and "ilasm" in names and "ildasm" in names
"#,
    );
}

#[test]
fn test_managed_tools_are_bundled_with_msvc() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
managed = [r for r in runtimes if r["name"] in ["csc", "ilasm", "ildasm"]]
len(managed) == 3 and all([r["bundled_with"] == "msvc" for r in managed])
"#,
    );
}

#[test]
fn test_msvc_runtime_has_cl_alias() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "msvc"][0]
"cl" in rt["aliases"]
"#,
    );
}

#[test]
fn test_msvc_runtime_has_system_paths() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "msvc"][0]
len(rt["system_paths"]) > 0
"#,
    );
}

#[test]
fn test_msvc_runtime_windows_only() {
    make_assert().is_true(
        r#"
load("provider.star", "runtimes")
rt = [r for r in runtimes if r["name"] == "msvc"][0]
rt["platform_constraint"]["os"] == ["windows"]
"#,
    );
}

// ── download_url logic ────────────────────────────────────────────────────────

#[test]
fn test_download_url_always_returns_none() {
    // msvc is installed via Visual Studio Installer, not direct download
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
url = download_url(ctx, "system")
url == None
"#,
        provider_star_prefix()
    ));
}

// ── fetch_versions logic ──────────────────────────────────────────────────────

#[test]
fn test_fetch_versions_returns_system_version() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
versions = fetch_versions(ctx)
len(versions) == 1 and versions[0]["version"] == "system"
"#,
        provider_star_prefix()
    ));
}

// ── environment logic ─────────────────────────────────────────────────────────

#[test]
fn test_environment_returns_empty_list() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""), install_dir = "C:\\msvc", vx_home = "C:\\Users\\user\\.vx")
env = environment(ctx, "system")
len(env) == 0
"#,
        provider_star_prefix()
    ));
}

// ── deps logic ────────────────────────────────────────────────────────────────

#[test]
fn test_deps_recommends_cmake_and_ninja() {
    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    a.is_true(&format!(
        r#"
{}
ctx = struct(platform = struct(os = "windows", arch = "x64", target = ""))
d = deps(ctx, "system")
dep_names = [dep["runtime"] for dep in d]
"cmake" in dep_names and "ninja" in dep_names
"#,
        provider_star_prefix()
    ));
}

// ── system_install logic ──────────────────────────────────────────────────────

#[test]
fn test_system_install_has_both_winget_and_choco() {
    make_assert().is_true(
        r#"
load("provider.star", "system_install")
strategies = system_install["strategies"]
managers = [s["manager"] for s in strategies]
"winget" in managers and "choco" in managers
"#,
    );
}

#[test]
fn test_system_install_includes_required_workloads() {
    make_assert().is_true(
        r#"
load("provider.star", "system_install")
strategies = system_install["strategies"]
args = " ".join([s.get("install_args", "") for s in strategies])
"Microsoft.VisualStudio.Workload.VCTools" in args and
"Microsoft.VisualStudio.Workload.ManagedDesktopBuildTools" in args and
"Microsoft.VisualStudio.Workload.NetCoreBuildTools" in args and
"Microsoft.VisualStudio.Workload.NativeGame" in args and
"Microsoft.VisualStudio.Workload.ManagedGame" in args
"#,
    );
}

// ── lint check ────────────────────────────────────────────────────────────────

#[test]
fn test_provider_star_lint_clean() {
    use starlark::analysis::AstModuleLint;
    use starlark::syntax::{AstModule, Dialect};
    use std::collections::HashSet;

    let ast = AstModule::parse(
        "provider.star",
        vx_provider_msvc::PROVIDER_STAR.to_string(),
        &Dialect::Standard,
    )
    .expect("provider.star should parse without errors");

    let known_globals: HashSet<String> = [
        "fetch_versions",
        "download_url",
        "environment",
        "post_install",
        "store_root",
        "get_execute_path",
        "deps",
        "system_install",
        "ctx",
        "name",
        "description",
        "homepage",
        "repository",
        "license",
        "ecosystem",
        "runtimes",
        "permissions",
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
