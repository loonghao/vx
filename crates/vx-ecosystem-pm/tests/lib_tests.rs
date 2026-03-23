//! Tests for vx-ecosystem-pm library functions
//!
//! Covers get_installer and get_preferred_installer functionality.

use vx_ecosystem_pm::{get_installer, get_preferred_installer};

// ============================================================================
// get_installer tests
// ============================================================================

#[test]
fn test_get_installer_npm() {
    let installer = get_installer("npm").expect("npm should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "npm");
}

#[test]
fn test_get_installer_node() {
    let installer = get_installer("node").expect("node should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "npm");
}

#[test]
fn test_get_installer_npx() {
    let installer = get_installer("npx").expect("npx should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "npm");
}

#[test]
fn test_get_installer_bun() {
    let installer = get_installer("bun").expect("bun should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "bun");
}

#[test]
fn test_get_installer_bunx() {
    let installer = get_installer("bunx").expect("bunx should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "bun");
}

#[test]
fn test_get_installer_pip() {
    let installer = get_installer("pip").expect("pip should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "pip");
}

#[test]
fn test_get_installer_uv() {
    let installer = get_installer("uv").expect("uv should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "uv");
}

#[test]
fn test_get_installer_uvx() {
    let installer = get_installer("uvx").expect("uvx should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "uvx");
}

#[test]
fn test_get_installer_cargo() {
    let installer = get_installer("cargo").expect("cargo should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "cargo");
}

#[test]
fn test_get_installer_go() {
    let installer = get_installer("go").expect("go should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "go");
}

#[test]
fn test_get_installer_gem() {
    let installer = get_installer("gem").expect("gem should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "gem");
}

#[test]
fn test_get_installer_yarn() {
    let installer = get_installer("yarn").expect("yarn should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "yarn");
}

#[test]
fn test_get_installer_pnpm() {
    let installer = get_installer("pnpm").expect("pnpm should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "pnpm");
}

#[test]
fn test_get_installer_dlx() {
    let installer = get_installer("dlx").expect("dlx should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "dlx");
}

#[test]
fn test_get_installer_deno() {
    let installer = get_installer("deno").expect("deno should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "deno");
}

#[test]
fn test_get_installer_dotnet_tool() {
    let installer = get_installer("dotnet-tool").expect("dotnet-tool should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "dotnet-tool");
}

#[test]
fn test_get_installer_dotnet_alias() {
    let installer = get_installer("dotnet").expect("dotnet should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "dotnet-tool");
}

#[test]
fn test_get_installer_jbang() {
    let installer = get_installer("jbang").expect("jbang should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "jbang");
}

#[test]
fn test_get_installer_java_alias() {
    let installer = get_installer("java").expect("java should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "jbang");
}

#[test]
fn test_get_installer_choco() {
    let installer = get_installer("choco").expect("choco should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "choco");
}

#[test]
fn test_get_installer_chocolatey_alias() {
    let installer = get_installer("chocolatey").expect("chocolatey should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "choco");
}

#[test]
fn test_get_installer_python_alias() {
    let installer = get_installer("python").expect("python should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "pip");
}

#[test]
fn test_get_installer_pypi_alias() {
    let installer = get_installer("pypi").expect("pypi should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "pip");
}

#[test]
fn test_get_installer_rust_alias() {
    let installer = get_installer("rust").expect("rust should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "cargo");
}

#[test]
fn test_get_installer_crates_alias() {
    let installer = get_installer("crates").expect("crates should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "cargo");
}

#[test]
fn test_get_installer_golang_alias() {
    let installer = get_installer("golang").expect("golang should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "go");
}

#[test]
fn test_get_installer_ruby_alias() {
    let installer = get_installer("ruby").expect("ruby should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "gem");
}

#[test]
fn test_get_installer_rubygems_alias() {
    let installer = get_installer("rubygems").expect("rubygems should be a valid ecosystem");
    assert_eq!(installer.ecosystem(), "gem");
}

#[test]
fn test_get_installer_case_insensitive() {
    // Ecosystem names should be case-insensitive
    let _ = get_installer("NPM").expect("NPM (uppercase) should work");
    let _ = get_installer("Pip").expect("Pip (mixed case) should work");
    let _ = get_installer("CARGO").expect("CARGO (uppercase) should work");
}

#[test]
fn test_get_installer_unsupported() {
    let result = get_installer("nonexistent-ecosystem-xyz");
    assert!(
        result.is_err(),
        "Unsupported ecosystem should return an error"
    );
}

// ============================================================================
// get_preferred_installer tests
// ============================================================================

#[test]
fn test_get_preferred_installer_python() {
    // Python prefers uv, falls back to pip
    let installer = get_preferred_installer("python").expect("python should work");
    let eco = installer.ecosystem();
    assert!(
        eco == "uv" || eco == "pip",
        "Python preferred installer should be uv or pip, got: {}",
        eco
    );
}

#[test]
fn test_get_preferred_installer_pip() {
    // pip also routes through the Python preference logic
    let installer = get_preferred_installer("pip").expect("pip should work");
    let eco = installer.ecosystem();
    assert!(
        eco == "uv" || eco == "pip",
        "pip preferred installer should be uv or pip, got: {}",
        eco
    );
}

#[test]
fn test_get_preferred_installer_pypi() {
    let installer = get_preferred_installer("pypi").expect("pypi should work");
    let eco = installer.ecosystem();
    assert!(
        eco == "uv" || eco == "pip",
        "pypi preferred installer should be uv or pip, got: {}",
        eco
    );
}

#[test]
fn test_get_preferred_installer_npm() {
    // npm prefers bun, falls back to npm
    let installer = get_preferred_installer("npm").expect("npm should work");
    let eco = installer.ecosystem();
    assert!(
        eco == "bun" || eco == "npm",
        "npm preferred installer should be bun or npm, got: {}",
        eco
    );
}

#[test]
fn test_get_preferred_installer_node() {
    // node routes through the npm preference logic
    let installer = get_preferred_installer("node").expect("node should work");
    let eco = installer.ecosystem();
    assert!(
        eco == "bun" || eco == "npm",
        "node preferred installer should be bun or npm, got: {}",
        eco
    );
}

#[test]
fn test_get_preferred_installer_npx() {
    // npx routes through the npm preference logic
    let installer = get_preferred_installer("npx").expect("npx should work");
    let eco = installer.ecosystem();
    assert!(
        eco == "bun" || eco == "npm",
        "npx preferred installer should be bun or npm, got: {}",
        eco
    );
}

#[test]
fn test_get_preferred_installer_other_ecosystems() {
    // Other ecosystems fall through to get_installer
    let cargo_installer = get_preferred_installer("cargo").expect("cargo should work");
    assert_eq!(cargo_installer.ecosystem(), "cargo");

    let go_installer = get_preferred_installer("go").expect("go should work");
    assert_eq!(go_installer.ecosystem(), "go");

    let gem_installer = get_preferred_installer("gem").expect("gem should work");
    assert_eq!(gem_installer.ecosystem(), "gem");
}

#[test]
fn test_get_preferred_installer_unsupported() {
    let result = get_preferred_installer("nonexistent-ecosystem-xyz");
    assert!(
        result.is_err(),
        "Unsupported ecosystem should return an error"
    );
}
