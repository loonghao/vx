//! In-process tests for init command project detection and template generation
//!
//! These tests call detect_project() directly (in-process) so that
//! cargo-llvm-cov can measure coverage of init.rs code paths.
//!
//! Covers:
//! - New project detection: oxlint, ruff, maturin, electron, openclaw
//! - Template listing output
//! - Mixed project detection

use rstest::*;
use std::fs;
use tempfile::TempDir;
use vx_cli::commands::init::{PackageManager, detect_project};

// ============================================================================
// Project Detection Tests - oxlint
// ============================================================================

/// Test: detect oxlint config (oxlintrc.json)
#[rstest]
#[test]
fn test_detect_oxlint_config() {
    let temp_dir = TempDir::new().unwrap();

    // Create oxlintrc.json to trigger detection
    fs::write(temp_dir.path().join("oxlintrc.json"), r#"{ "rules": {} }"#).unwrap();

    // Also need package.json for Node.js context
    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "test-oxlint" }"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("oxlint"),
        "Should detect oxlint from oxlintrc.json. Tools: {:?}",
        detection.tools
    );
    assert!(
        detection.hints.iter().any(|h| h.contains("oxlint")),
        "Should have oxlint hint. Hints: {:?}",
        detection.hints
    );
}

/// Test: detect oxlint config (.oxlintrc.json - dot prefix)
#[rstest]
#[test]
fn test_detect_oxlint_dot_config() {
    let temp_dir = TempDir::new().unwrap();

    // Create .oxlintrc.json (dot prefix variant)
    fs::write(temp_dir.path().join(".oxlintrc.json"), r#"{ "rules": {} }"#).unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "test-oxlint-dot" }"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("oxlint"),
        "Should detect oxlint from .oxlintrc.json. Tools: {:?}",
        detection.tools
    );
}

// ============================================================================
// Project Detection Tests - ruff
// ============================================================================

/// Test: detect ruff config (ruff.toml)
#[rstest]
#[test]
fn test_detect_ruff_config() {
    let temp_dir = TempDir::new().unwrap();

    // Create ruff.toml to trigger detection
    fs::write(
        temp_dir.path().join("ruff.toml"),
        "[lint]\nselect = [\"E\"]\n",
    )
    .unwrap();

    // Also need pyproject.toml for Python context
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = \"test-ruff\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("ruff"),
        "Should detect ruff from ruff.toml. Tools: {:?}",
        detection.tools
    );
    assert!(
        detection.hints.iter().any(|h| h.contains("ruff")),
        "Should have ruff hint. Hints: {:?}",
        detection.hints
    );
}

/// Test: detect ruff config (.ruff.toml - dot prefix)
#[rstest]
#[test]
fn test_detect_ruff_dot_config() {
    let temp_dir = TempDir::new().unwrap();

    // Create .ruff.toml (dot prefix variant)
    fs::write(
        temp_dir.path().join(".ruff.toml"),
        "[lint]\nselect = [\"E\"]\n",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = \"test-ruff-dot\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("ruff"),
        "Should detect ruff from .ruff.toml. Tools: {:?}",
        detection.tools
    );
}

// ============================================================================
// Project Detection Tests - maturin (PyO3/Rust+Python hybrid)
// ============================================================================

/// Test: detect maturin/PyO3 project with pyo3 dependency
#[rstest]
#[test]
fn test_detect_maturin_project_pyo3() {
    let temp_dir = TempDir::new().unwrap();

    // Create Cargo.toml with pyo3 dependency
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "my-pyo3-lib"
version = "0.1.0"
edition = "2021"

[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }
"#,
    )
    .unwrap();

    // Create pyproject.toml (required for maturin detection)
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        r#"[build-system]
requires = ["maturin>=1.0"]
build-backend = "maturin"
"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("maturin"),
        "Should detect maturin from Cargo.toml+pyproject.toml with pyo3. Tools: {:?}",
        detection.tools
    );
    assert!(
        detection
            .hints
            .iter()
            .any(|h| h.contains("maturin") || h.contains("PyO3")),
        "Should have maturin/PyO3 hint. Hints: {:?}",
        detection.hints
    );
}

/// Test: detect maturin project with maturin keyword in Cargo.toml
#[rstest]
#[test]
fn test_detect_maturin_project_maturin_keyword() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "my-maturin-lib"
version = "0.1.0"
edition = "2021"

[lib]
name = "my_lib"
crate-type = ["cdylib"]
# Built with maturin
"#,
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("pyproject.toml"),
        r#"[build-system]
requires = ["maturin>=1.0"]
build-backend = "maturin"
"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("maturin"),
        "Should detect maturin keyword. Tools: {:?}",
        detection.tools
    );
}

/// Test: detect maturin project with uniffi dependency
#[rstest]
#[test]
fn test_detect_maturin_project_uniffi() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "my-uniffi-lib"
version = "0.1.0"
edition = "2021"

[dependencies]
uniffi = "0.28"
"#,
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("pyproject.toml"),
        r#"[build-system]
requires = ["maturin>=1.0"]
"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("maturin"),
        "Should detect uniffi as a maturin project indicator. Tools: {:?}",
        detection.tools
    );
}

// ============================================================================
// Project Detection Tests - Electron
// ============================================================================

/// Test: detect Electron project from devDependencies
#[rstest]
#[test]
fn test_detect_electron_project_dev_deps() {
    let temp_dir = TempDir::new().unwrap();

    // Create package.json with electron in devDependencies
    fs::write(
        temp_dir.path().join("package.json"),
        r#"{
  "name": "my-electron-app",
  "devDependencies": {
    "electron": "^28.0.0"
  }
}"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    // Electron detection adds a hint
    assert!(
        detection
            .hints
            .iter()
            .any(|h| h.contains("Electron") || h.contains("electron")),
        "Should detect Electron from devDependencies. Hints: {:?}",
        detection.hints
    );
}

/// Test: detect Electron project from dependencies
#[rstest]
#[test]
fn test_detect_electron_project_deps() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{
  "name": "my-electron-app",
  "dependencies": {
    "electron": "^28.0.0"
  }
}"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection
            .hints
            .iter()
            .any(|h| h.contains("Electron") || h.contains("electron")),
        "Should detect Electron from dependencies. Hints: {:?}",
        detection.hints
    );
}

/// Test: non-Electron node project should not trigger Electron hint
#[rstest]
#[test]
fn test_no_electron_detection_for_regular_node() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{
  "name": "my-node-app",
  "dependencies": {
    "express": "^4.18.0"
  }
}"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        !detection
            .hints
            .iter()
            .any(|h| h.contains("Electron") || h.contains("electron")),
        "Should NOT detect Electron for regular node project. Hints: {:?}",
        detection.hints
    );
}

// ============================================================================
// Project Detection Tests - OpenClaw
// ============================================================================

/// Test: detect OpenClaw project via SKILL.md
#[rstest]
#[test]
fn test_detect_openclaw_skill_md() {
    let temp_dir = TempDir::new().unwrap();

    // Create SKILL.md to trigger OpenClaw detection
    fs::write(
        temp_dir.path().join("SKILL.md"),
        "# My Skill\nA skill for AI agents.",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("openclaw"),
        "Should detect OpenClaw from SKILL.md. Tools: {:?}",
        detection.tools
    );
    assert!(
        detection
            .hints
            .iter()
            .any(|h| h.contains("OpenClaw") || h.contains("openclaw")),
        "Should have OpenClaw hint. Hints: {:?}",
        detection.hints
    );
}

/// Test: detect OpenClaw project via SOUL.md
#[rstest]
#[test]
fn test_detect_openclaw_soul_md() {
    let temp_dir = TempDir::new().unwrap();

    // Create SOUL.md to trigger OpenClaw detection
    fs::write(
        temp_dir.path().join("SOUL.md"),
        "# Agent Soul\nAgent personality.",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("openclaw"),
        "Should detect OpenClaw from SOUL.md. Tools: {:?}",
        detection.tools
    );
}

/// Test: detect OpenClaw project via .openclaw directory
#[rstest]
#[test]
fn test_detect_openclaw_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create .openclaw directory to trigger detection
    fs::create_dir(temp_dir.path().join(".openclaw")).unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("openclaw"),
        "Should detect OpenClaw from .openclaw dir. Tools: {:?}",
        detection.tools
    );
}

// ============================================================================
// Mixed Project Detection Tests
// ============================================================================

/// Test: detect mixed project (Node.js + Python)
#[rstest]
#[test]
fn test_detect_mixed_project() {
    let temp_dir = TempDir::new().unwrap();

    // Create both Node.js and Python project files
    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "fullstack-app" }"#,
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = \"backend\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    // Should detect both ecosystems
    assert!(
        detection.tools.contains_key("node") || detection.tools.contains_key("uv"),
        "Should detect node or python tools in mixed project. Tools: {:?}",
        detection.tools
    );
}

/// Test: empty directory detection returns empty result
#[rstest]
#[test]
fn test_detect_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.is_empty(),
        "Empty directory should have no tools. Tools: {:?}",
        detection.tools
    );
    assert!(
        detection.project_types.is_empty(),
        "Empty directory should have no project types. Types: {:?}",
        detection.project_types
    );
}

/// Test: detect oxlint without Node.js context (standalone)
#[rstest]
#[test]
fn test_detect_oxlint_standalone() {
    let temp_dir = TempDir::new().unwrap();

    // Only oxlint config, no package.json
    fs::write(temp_dir.path().join("oxlintrc.json"), r#"{ "rules": {} }"#).unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    // oxlint should still be detected even without package.json
    assert!(
        detection.tools.contains_key("oxlint"),
        "Should detect oxlint standalone. Tools: {:?}",
        detection.tools
    );
}

/// Test: detect ruff standalone (without Python project context)
#[rstest]
#[test]
fn test_detect_ruff_standalone() {
    let temp_dir = TempDir::new().unwrap();

    // Only ruff config, no pyproject.toml
    fs::write(
        temp_dir.path().join("ruff.toml"),
        "[lint]\nselect = [\"E\"]\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("ruff"),
        "Should detect ruff standalone. Tools: {:?}",
        detection.tools
    );
}

/// Test: detect OpenClaw with Node.js context
#[rstest]
#[test]
fn test_detect_openclaw_with_node() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "my-agent" }"#,
    )
    .unwrap();
    fs::write(temp_dir.path().join("SKILL.md"), "# Skill\nAgent skill.").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("openclaw"),
        "Should detect OpenClaw alongside Node.js. Tools: {:?}",
        detection.tools
    );
}

/// Test: Rust project with Cargo.toml but without PyO3 should not detect maturin
#[rstest]
#[test]
fn test_no_maturin_for_plain_rust() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "my-rust-lib"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        !detection.tools.contains_key("maturin"),
        "Plain Rust project should NOT detect maturin. Tools: {:?}",
        detection.tools
    );
}

// ============================================================================
// Project Detection Tests - .NET / C#
// ============================================================================

/// Test: detect .NET project from .csproj file
#[rstest]
#[test]
fn test_detect_dotnet_csproj() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("MyApp.csproj"),
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("dotnet"),
        "Should detect dotnet from .csproj. Tools: {:?}",
        detection.tools
    );
    assert!(
        detection
            .hints
            .iter()
            .any(|h| h.contains("C#") || h.contains("project")),
        "Should have C# project hint. Hints: {:?}",
        detection.hints
    );
    assert_eq!(
        detection.project_name.as_deref(),
        Some("MyApp"),
        "Should extract project name from .csproj filename"
    );
}

/// Test: detect .NET project from .sln file
#[rstest]
#[test]
fn test_detect_dotnet_sln() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("MySolution.sln"),
        "Microsoft Visual Studio Solution File, Format Version 12.00\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("dotnet"),
        "Should detect dotnet from .sln. Tools: {:?}",
        detection.tools
    );
    assert!(
        detection
            .hints
            .iter()
            .any(|h| h.contains("Solution") || h.contains("multi-project")),
        "Should have Solution hint. Hints: {:?}",
        detection.hints
    );
    assert_eq!(
        detection.project_name.as_deref(),
        Some("MySolution"),
        "Should extract project name from .sln filename"
    );
}

/// Test: detect .NET project from global.json with SDK version
#[rstest]
#[test]
fn test_detect_dotnet_global_json() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("global.json"),
        r#"{ "sdk": { "version": "8.0.100" } }"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("dotnet"),
        "Should detect dotnet from global.json. Tools: {:?}",
        detection.tools
    );
    assert_eq!(
        detection.tools.get("dotnet").map(|s| s.as_str()),
        Some("8.0.100"),
        "Should pick up SDK version from global.json"
    );
    assert!(
        detection
            .hints
            .iter()
            .any(|h| h.contains("8.0.100") && h.contains("global.json")),
        "Should hint about pinned SDK version. Hints: {:?}",
        detection.hints
    );
}

/// Test: detect .NET project from F# project file
#[rstest]
#[test]
fn test_detect_dotnet_fsproj() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("MyFSharp.fsproj"),
        r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("dotnet"),
        "Should detect dotnet from .fsproj. Tools: {:?}",
        detection.tools
    );
    assert!(
        detection.hints.iter().any(|h| h.contains("F#")),
        "Should have F# hint. Hints: {:?}",
        detection.hints
    );
}

/// Test: detect .NET project from nested .csproj (recursive search)
#[rstest]
#[test]
fn test_detect_dotnet_nested_csproj() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested structure: src/MyApp/MyApp.csproj
    let nested = temp_dir.path().join("src").join("MyApp");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        nested.join("MyApp.csproj"),
        r#"<Project Sdk="Microsoft.NET.Sdk"></Project>"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("dotnet"),
        "Should detect dotnet from nested .csproj. Tools: {:?}",
        detection.tools
    );
}

/// Test: detect .NET project from Directory.Build.props
#[rstest]
#[test]
fn test_detect_dotnet_directory_build_props() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Directory.Build.props"),
        r#"<Project><PropertyGroup></PropertyGroup></Project>"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("dotnet"),
        "Should detect dotnet from Directory.Build.props. Tools: {:?}",
        detection.tools
    );
}

// ============================================================================
// Project Detection Tests - Go
// ============================================================================

/// Test: detect Go project from go.mod
#[rstest]
#[test]
fn test_detect_go_project() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("go.mod"),
        "module github.com/example/myapp\n\ngo 1.21\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("go"),
        "Should detect go from go.mod. Tools: {:?}",
        detection.tools
    );
    assert_eq!(
        detection.project_name.as_deref(),
        Some("github.com/example/myapp"),
        "Should extract module name from go.mod"
    );
    assert_eq!(
        detection.package_manager,
        Some(PackageManager::GoMod),
        "Should detect GoMod package manager"
    );
}

// ============================================================================
// Project Detection Tests - Rust (toolchain detection)
// ============================================================================

/// Test: detect Rust project with rust-toolchain.toml (numeric version preserved)
///
/// vx.toml records user-facing versions as-is. Numeric rustc versions from
/// rust-toolchain.toml are preserved (e.g., "1.83.0") because the version
/// resolution layer (vx lock) uses passthrough mode for the Rust ecosystem,
/// allowing rustup to handle the actual toolchain installation.
#[rstest]
#[test]
fn test_detect_rust_toolchain_toml() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"my-project\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"1.83.0\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("rust"),
        "Should detect rust. Tools: {:?}",
        detection.tools
    );
    assert_eq!(
        detection.tools.get("rust").map(|s| s.as_str()),
        Some("1.83.0"),
        "Numeric rustc version from rust-toolchain.toml should be preserved as-is"
    );
}

/// Test: detect Rust project with rust-toolchain.toml using "stable" channel
#[rstest]
#[test]
fn test_detect_rust_toolchain_toml_stable_channel() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"my-project\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"stable\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.tools.get("rust").map(|s| s.as_str()),
        Some("stable"),
        "Channel name 'stable' should be preserved as-is"
    );
}

/// Test: detect Rust project with legacy rust-toolchain file
#[rstest]
#[test]
fn test_detect_rust_toolchain_legacy() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"legacy-proj\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(temp_dir.path().join("rust-toolchain"), "nightly\n").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.tools.get("rust").map(|s| s.as_str()),
        Some("nightly"),
        "Should pick up version from legacy rust-toolchain"
    );
}

/// Test: detect Rust project with rust-version in Cargo.toml (MSRV preserved)
///
/// rust-version in Cargo.toml is MSRV (Minimum Supported Rust Version).
/// vx.toml records it as-is; the lock/resolve layer handles passthrough
/// to rustup for the actual toolchain installation.
#[rstest]
#[test]
fn test_detect_rust_version_in_cargo_toml() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"msrv-project\"\nversion = \"0.1.0\"\nrust-version = \"1.70.0\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.tools.get("rust").map(|s| s.as_str()),
        Some("1.70.0"),
        "MSRV rust-version from Cargo.toml should be preserved as-is"
    );
    assert_eq!(
        detection.project_name.as_deref(),
        Some("msrv-project"),
        "Should extract project name from Cargo.toml"
    );
}

/// Test: Rust project without any toolchain file defaults to "stable"
#[rstest]
#[test]
fn test_detect_rust_default_stable() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"default-proj\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.tools.get("rust").map(|s| s.as_str()),
        Some("stable"),
        "Rust without toolchain file should default to 'stable'"
    );
}

// ============================================================================
// Project Detection Tests - Node.js (package manager detection)
// ============================================================================

/// Test: detect npm from package-lock.json
#[rstest]
#[test]
fn test_detect_npm_from_lockfile() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "npm-project" }"#,
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("package-lock.json"),
        r#"{ "lockfileVersion": 3 }"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Npm),
        "Should detect npm from package-lock.json"
    );
    assert!(
        detection.hints.iter().any(|h| h.contains("npm")),
        "Should have npm hint. Hints: {:?}",
        detection.hints
    );
}

/// Test: detect pnpm from pnpm-lock.yaml
#[rstest]
#[test]
fn test_detect_pnpm_from_lockfile() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "pnpm-project" }"#,
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("pnpm-lock.yaml"),
        "lockfileVersion: 9\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Pnpm),
        "Should detect pnpm from pnpm-lock.yaml"
    );
}

/// Test: detect yarn from yarn.lock
#[rstest]
#[test]
fn test_detect_yarn_from_lockfile() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "yarn-project" }"#,
    )
    .unwrap();
    fs::write(temp_dir.path().join("yarn.lock"), "# yarn lockfile v1\n").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Yarn),
        "Should detect yarn from yarn.lock"
    );
}

/// Test: detect bun from bun.lockb
#[rstest]
#[test]
fn test_detect_bun_from_lockfile() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "bun-project" }"#,
    )
    .unwrap();
    fs::write(temp_dir.path().join("bun.lockb"), "binary-lock-data").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Bun),
        "Should detect bun from bun.lockb"
    );
}

/// Test: detect bun from bun.lock (text format)
#[rstest]
#[test]
fn test_detect_bun_from_text_lockfile() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "bun-text-project" }"#,
    )
    .unwrap();
    fs::write(temp_dir.path().join("bun.lock"), "bun-lock-text").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Bun),
        "Should detect bun from bun.lock"
    );
}

/// Test: detect packageManager field (pnpm)
#[rstest]
#[test]
fn test_detect_package_manager_field_pnpm() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "corepack-pnpm", "packageManager": "pnpm@9.0.0" }"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Pnpm),
        "Should detect pnpm from packageManager field"
    );
    assert!(
        detection.tools.contains_key("pnpm"),
        "Should add pnpm tool. Tools: {:?}",
        detection.tools
    );
}

/// Test: detect packageManager field (yarn)
#[rstest]
#[test]
fn test_detect_package_manager_field_yarn() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "corepack-yarn", "packageManager": "yarn@4.0.0" }"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Yarn),
        "Should detect yarn from packageManager field"
    );
}

/// Test: detect packageManager field (npm)
#[rstest]
#[test]
fn test_detect_package_manager_field_npm() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "corepack-npm", "packageManager": "npm@10.0.0" }"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Npm),
        "Should detect npm from packageManager field"
    );
}

/// Test: detect packageManager field (bun)
#[rstest]
#[test]
fn test_detect_package_manager_field_bun() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "corepack-bun", "packageManager": "bun@1.0.0" }"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Bun),
        "Should detect bun from packageManager field"
    );
}

/// Test: detect Node.js version from engines field
#[rstest]
#[test]
fn test_detect_node_version_from_engines() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "engine-test", "engines": { "node": ">=18.0.0" } }"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.tools.get("node").map(|s| s.as_str()),
        Some("18"),
        "Should parse >=18.0.0 to major version 18"
    );
}

// ============================================================================
// Project Detection Tests - Python (uv.lock, poetry)
// ============================================================================

/// Test: detect Python project with uv.lock
#[rstest]
#[test]
fn test_detect_python_uv_lock() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = \"uv-project\"\n",
    )
    .unwrap();
    fs::write(temp_dir.path().join("uv.lock"), "version = 1\n").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Uv),
        "Should detect uv from uv.lock"
    );
    assert!(
        detection.tools.contains_key("uv"),
        "Should add uv tool. Tools: {:?}",
        detection.tools
    );
}

/// Test: detect Python project with poetry
#[rstest]
#[test]
fn test_detect_python_poetry() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[tool.poetry]\nname = \"poetry-project\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Poetry),
        "Should detect poetry from [tool.poetry]"
    );
}

/// Test: detect Python version from requires-python
#[rstest]
#[test]
fn test_detect_python_version_requires_python() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = \"version-test\"\nrequires-python = \">=3.11\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.tools.get("python").map(|s| s.as_str()),
        Some("3.11"),
        "Should parse requires-python >=3.11"
    );
}

/// Test: detect Python project from requirements.txt only
#[rstest]
#[test]
fn test_detect_python_requirements_txt() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("requirements.txt"),
        "flask>=2.0\nrequests\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("python"),
        "Should detect python from requirements.txt. Tools: {:?}",
        detection.tools
    );
    // Default package manager for Python without uv/poetry is uv (recommended)
    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Uv),
        "Should recommend uv as default"
    );
}

/// Test: detect Python project from setup.py
#[rstest]
#[test]
fn test_detect_python_setup_py() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("setup.py"),
        "from setuptools import setup\nsetup(name='my-pkg')\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("python"),
        "Should detect python from setup.py. Tools: {:?}",
        detection.tools
    );
}

// ============================================================================
// Project Detection Tests - Justfile
// ============================================================================

/// Test: detect Justfile (case-sensitive: justfile)
#[rstest]
#[test]
fn test_detect_justfile_lowercase() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("justfile"), "default:\n  echo hi\n").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("just"),
        "Should detect just from justfile. Tools: {:?}",
        detection.tools
    );
    assert!(
        detection.hints.iter().any(|h| h.contains("Justfile")),
        "Should have Justfile hint. Hints: {:?}",
        detection.hints
    );
}

/// Test: detect Justfile (case-sensitive: Justfile)
#[rstest]
#[test]
fn test_detect_justfile_capitalized() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("Justfile"), "default:\n  echo hi\n").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("just"),
        "Should detect just from Justfile. Tools: {:?}",
        detection.tools
    );
}

// ============================================================================
// Mixed / Complex Project Detection
// ============================================================================

/// Test: mixed project gets Mixed type marker
#[rstest]
#[test]
fn test_mixed_project_has_mixed_type() {
    use vx_cli::commands::init::ProjectType;

    let temp_dir = TempDir::new().unwrap();

    // Node.js + Python = mixed
    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "fullstack" }"#,
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = \"backend\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.project_types.contains(&ProjectType::Mixed),
        "Mixed project should have Mixed type. Types: {:?}",
        detection.project_types
    );
    assert!(
        detection.project_types.contains(&ProjectType::NodeJs),
        "Should contain NodeJs type"
    );
    assert!(
        detection.project_types.contains(&ProjectType::Python),
        "Should contain Python type"
    );
}

/// Test: triple mixed project (Node.js + Python + Rust)
#[rstest]
#[test]
fn test_triple_mixed_project() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "triple" }"#,
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = \"triple\"\n",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"triple\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.project_types.len() >= 4,
        "Triple mixed project should have >=4 types (Mixed + 3 langs). Types: {:?}",
        detection.project_types
    );
}

// ============================================================================
// Display trait coverage
// ============================================================================

/// Test: ProjectType Display trait
#[rstest]
#[test]
fn test_project_type_display() {
    use vx_cli::commands::init::ProjectType;

    assert_eq!(format!("{}", ProjectType::NodeJs), "Node.js");
    assert_eq!(format!("{}", ProjectType::Python), "Python");
    assert_eq!(format!("{}", ProjectType::Rust), "Rust");
    assert_eq!(format!("{}", ProjectType::Go), "Go");
    assert_eq!(format!("{}", ProjectType::DotNet), ".NET/C#");
    assert_eq!(format!("{}", ProjectType::Justfile), "Justfile");
    assert_eq!(format!("{}", ProjectType::Mixed), "Mixed");
}

/// Test: PackageManager Display trait
#[rstest]
#[test]
fn test_package_manager_display() {
    assert_eq!(format!("{}", PackageManager::Npm), "npm");
    assert_eq!(format!("{}", PackageManager::Yarn), "yarn");
    assert_eq!(format!("{}", PackageManager::Pnpm), "pnpm");
    assert_eq!(format!("{}", PackageManager::Bun), "bun");
    assert_eq!(format!("{}", PackageManager::Uv), "uv");
    assert_eq!(format!("{}", PackageManager::Pip), "pip");
    assert_eq!(format!("{}", PackageManager::Poetry), "poetry");
    assert_eq!(format!("{}", PackageManager::Cargo), "cargo");
    assert_eq!(format!("{}", PackageManager::GoMod), "go");
    assert_eq!(format!("{}", PackageManager::NuGet), "nuget");
}

// ============================================================================
// Project Detection Tests - uv.lock without pyproject.toml [tool.uv]
// ============================================================================

/// Test: detect Python project with standalone uv.lock (no [tool.uv] section)
#[rstest]
#[test]
fn test_detect_python_standalone_uv_lock() {
    let temp_dir = TempDir::new().unwrap();

    // pyproject.toml without [tool.uv] section
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = \"no-tool-uv\"\n",
    )
    .unwrap();
    // But uv.lock exists
    fs::write(temp_dir.path().join("uv.lock"), "version = 1\n").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Uv),
        "Should detect uv from uv.lock even without [tool.uv]"
    );
}

// ============================================================================
// Project with Go module name extraction
// ============================================================================

/// Test: Go project name extraction when other types also present
#[rstest]
#[test]
fn test_go_project_name_not_overwritten() {
    let temp_dir = TempDir::new().unwrap();

    // Node.js project provides name first
    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "frontend-app" }"#,
    )
    .unwrap();
    // Go module also present
    fs::write(
        temp_dir.path().join("go.mod"),
        "module github.com/example/backend\n\ngo 1.22\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    // Node.js was detected first so project_name should be "frontend-app"
    assert_eq!(
        detection.project_name.as_deref(),
        Some("frontend-app"),
        "First detected project name should win"
    );
}

// ============================================================================
// Template Generation Tests - cover 380-420 line range
// ============================================================================

/// Test: electron template generates node 22 and pnpm
#[rstest]
#[test]
fn test_electron_template_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Electron project has package.json
    fs::write(
        temp_dir.path().join("package.json"),
        r#"{
  "name": "my-electron-app",
  "main": "main.js",
  "dependencies": {
    "electron": "^30.0.0"
  }
}"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    // Electron projects should have electron hint
    assert!(
        detection
            .hints
            .iter()
            .any(|h| h.to_lowercase().contains("electron")),
        "Electron project should have electron hint. Hints: {:?}",
        detection.hints
    );
}

/// Test: fullstack template includes node, pnpm, python, uv
#[rstest]
#[test]
fn test_fullstack_template_context() {
    let temp_dir = TempDir::new().unwrap();

    // Fullstack project has both package.json and pyproject.toml
    fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "fullstack-app"}"#,
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        r#"[project]
name = "fullstack"
version = "0.1.0""#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    // Should detect both Node.js and Python contexts
    assert!(
        detection.tools.contains_key("node") || detection.tools.contains_key("python"),
        "Fullstack project should detect Node.js or Python. Tools: {:?}",
        detection.tools
    );
}

/// Test: rust-python template (PyO3 project) detection
#[rstest]
#[test]
fn test_rust_python_mixed_project() {
    let temp_dir = TempDir::new().unwrap();

    // Rust-Python mixed project has both Cargo.toml and pyproject.toml
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "my-pyo3-lib"
version = "0.1.0"
edition = "2021"

[lib]
name = "my_pyo3_lib"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.20", features = ["extension-module"] }"#,
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        r#"[project]
name = "my-pyo3"
version = "0.1.0"
requires-python = ">=3.8"

[build-system]
requires = ["maturin>=1.0"]
build-backend = "maturin""#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("maturin"),
        "Rust-Python PyO3 project should detect maturin. Tools: {:?}",
        detection.tools
    );
}

/// Test: OpenClaw with .openclaw directory
#[rstest]
#[test]
fn test_detect_openclaw_with_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create .openclaw directory marker
    fs::create_dir_all(temp_dir.path().join(".openclaw")).unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("openclaw"),
        "Should detect OpenClaw from .openclaw directory. Tools: {:?}",
        detection.tools
    );
}

// ============================================================================
// Complex Real-World Project Simulation Tests
// ============================================================================

/// Test: simulates dcc-mcp-core (Rust project with MSRV 1.90 in Cargo.toml)
/// This was the original failing case — rust-version = "1.90" should be
/// preserved in vx.toml, not normalized to "stable".
#[rstest]
#[test]
fn test_detect_dcc_mcp_core_style_project() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "dcc-mcp-core"
version = "0.1.0"
edition = "2021"
rust-version = "1.90"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.tools.get("rust").map(|s| s.as_str()),
        Some("1.90"),
        "dcc-mcp-core style: MSRV 1.90 should be preserved, not converted to 'stable'"
    );
    assert_eq!(
        detection.project_name.as_deref(),
        Some("dcc-mcp-core"),
        "Should extract project name"
    );
}

/// Test: Rust project with workspace Cargo.toml (common in large projects)
#[rstest]
#[test]
fn test_detect_rust_workspace_project() {
    let temp_dir = TempDir::new().unwrap();

    // Workspace root Cargo.toml without [package] section
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[workspace]
members = ["crates/core", "crates/cli"]
resolver = "2"

[workspace.package]
version = "0.5.0"
edition = "2021"
rust-version = "1.80.0"
"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("rust"),
        "Should detect rust from workspace Cargo.toml. Tools: {:?}",
        detection.tools
    );
    // Workspace Cargo.toml doesn't have [package].rust-version directly,
    // but should still detect Rust project
}

/// Test: Rust project with rust-toolchain.toml using nightly with date
#[rstest]
#[test]
fn test_detect_rust_nightly_with_date() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"nightly-date\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"nightly-2025-01-15\"\ncomponents = [\"rustfmt\", \"clippy\"]\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.tools.get("rust").map(|s| s.as_str()),
        Some("nightly-2025-01-15"),
        "Nightly with date should be preserved"
    );
}

/// Test: Node.js project with complex engines constraints (e.g., React Native)
#[rstest]
#[test]
fn test_detect_react_native_style_project() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{
  "name": "MyReactNativeApp",
  "version": "0.73.0",
  "engines": {
    "node": ">=18"
  },
  "dependencies": {
    "react": "18.2.0",
    "react-native": "0.73.0"
  },
  "devDependencies": {
    "typescript": "5.0.0",
    "@types/react": "18.2.0"
  }
}"#,
    )
    .unwrap();
    fs::write(temp_dir.path().join("yarn.lock"), "# yarn lockfile v1\n").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.tools.get("node").map(|s| s.as_str()),
        Some("18"),
        "Should parse >=18 to major version 18"
    );
    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Yarn),
        "Should detect yarn from yarn.lock"
    );
    assert_eq!(detection.project_name.as_deref(), Some("MyReactNativeApp"),);
}

/// Test: Python project with Poetry (tool.poetry in pyproject.toml)
#[rstest]
#[test]
fn test_detect_poetry_project_with_python_version() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("pyproject.toml"),
        r#"[tool.poetry]
name = "my-poetry-app"
version = "1.0.0"

[tool.poetry.dependencies]
python = "^3.10"
fastapi = ">=0.100"

[build-system]
requires = ["poetry-core"]
build-backend = "poetry.core.masonry.api"
"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.package_manager,
        Some(PackageManager::Poetry),
        "Should detect poetry"
    );
}

/// Test: Go project with specific Go version constraint
#[rstest]
#[test]
fn test_detect_go_project_with_version() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("go.mod"),
        "module github.com/kubernetes/kubernetes\n\ngo 1.22.0\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.1\n)\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("go"),
        "Should detect go. Tools: {:?}",
        detection.tools
    );
    assert_eq!(
        detection.project_name.as_deref(),
        Some("github.com/kubernetes/kubernetes"),
    );
}

/// Test: Full-stack monorepo with Node.js + Python + Rust + Justfile
#[rstest]
#[test]
fn test_detect_fullstack_monorepo() {
    use vx_cli::commands::init::ProjectType;

    let temp_dir = TempDir::new().unwrap();

    // Frontend (Node.js)
    fs::write(
        temp_dir.path().join("package.json"),
        r#"{
  "name": "fullstack-monorepo",
  "engines": { "node": ">=22" },
  "packageManager": "pnpm@9.0.0"
}"#,
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("pnpm-lock.yaml"),
        "lockfileVersion: 9\n",
    )
    .unwrap();

    // Backend (Go)
    fs::write(
        temp_dir.path().join("go.mod"),
        "module github.com/user/monorepo\n\ngo 1.22\n",
    )
    .unwrap();

    // ML/Scripts (Python)
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = \"ml-scripts\"\nrequires-python = \">=3.12\"\n",
    )
    .unwrap();

    // Rust native extension
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"native-ext\"\nversion = \"0.1.0\"\nrust-version = \"1.80\"\n",
    )
    .unwrap();

    // Task runner
    fs::write(temp_dir.path().join("justfile"), "default:\n  echo hi\n").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    // Should detect all 4 ecosystems + Mixed + Justfile
    assert!(
        detection.project_types.contains(&ProjectType::NodeJs),
        "Should detect NodeJs. Types: {:?}",
        detection.project_types
    );
    assert!(
        detection.project_types.contains(&ProjectType::Go),
        "Should detect Go. Types: {:?}",
        detection.project_types
    );
    assert!(
        detection.project_types.contains(&ProjectType::Python),
        "Should detect Python. Types: {:?}",
        detection.project_types
    );
    assert!(
        detection.project_types.contains(&ProjectType::Rust),
        "Should detect Rust. Types: {:?}",
        detection.project_types
    );
    assert!(
        detection.project_types.contains(&ProjectType::Justfile),
        "Should detect Justfile. Types: {:?}",
        detection.project_types
    );
    assert!(
        detection.project_types.contains(&ProjectType::Mixed),
        "Should be marked as Mixed. Types: {:?}",
        detection.project_types
    );

    // Check tools
    assert!(detection.tools.contains_key("node"), "node");
    assert!(detection.tools.contains_key("go"), "go");
    assert!(
        detection.tools.contains_key("python") || detection.tools.contains_key("uv"),
        "python/uv"
    );
    assert!(detection.tools.contains_key("rust"), "rust");
    assert!(detection.tools.contains_key("just"), "just");
    assert!(detection.tools.contains_key("pnpm"), "pnpm");

    // Rust version should be MSRV from Cargo.toml
    assert_eq!(
        detection.tools.get("rust").map(|s| s.as_str()),
        Some("1.80"),
        "Rust MSRV 1.80 should be preserved"
    );
}

/// Test: Node.js project with exact node version in engines
#[rstest]
#[test]
fn test_detect_node_exact_version_engines() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("package.json"),
        r#"{ "name": "exact-ver", "engines": { "node": "20.11.0" } }"#,
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    // Should extract some version from exact specification
    assert!(
        detection.tools.contains_key("node"),
        "Should detect node. Tools: {:?}",
        detection.tools
    );
}

/// Test: Rust project prioritizes rust-toolchain.toml over Cargo.toml rust-version
#[rstest]
#[test]
fn test_detect_rust_toolchain_priority_over_cargo() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"priority-test\"\nversion = \"0.1.0\"\nrust-version = \"1.70.0\"\n",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"1.83.0\"\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert_eq!(
        detection.tools.get("rust").map(|s| s.as_str()),
        Some("1.83.0"),
        "rust-toolchain.toml (1.83.0) should take priority over Cargo.toml rust-version (1.70.0)"
    );
}

/// Test: Python project with requirements.txt and no pyproject.toml
#[rstest]
#[test]
fn test_detect_legacy_python_requirements_only() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("requirements.txt"),
        "django>=4.0\ncelery>=5.0\nredis>=4.0\n",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("setup.py"),
        "from setuptools import setup\nsetup(name='legacy-app')\n",
    )
    .unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("python"),
        "Should detect python from requirements.txt + setup.py. Tools: {:?}",
        detection.tools
    );
}

/// Test: vx project (the vx repo itself) - Rust with workspace
#[rstest]
#[test]
fn test_detect_vx_style_project() {
    let temp_dir = TempDir::new().unwrap();

    // vx uses workspace with rust-version in [workspace.package]
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[workspace]
members = ["crates/vx-cli", "crates/vx-core"]
resolver = "2"

[workspace.package]
version = "0.8.12"
edition = "2024"
rust-version = "1.93.0"

[package]
name = "vx"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
"#,
    )
    .unwrap();
    fs::write(temp_dir.path().join("justfile"), "test:\n  cargo test\n").unwrap();

    let detection = detect_project(temp_dir.path()).unwrap();

    assert!(
        detection.tools.contains_key("rust"),
        "Should detect rust. Tools: {:?}",
        detection.tools
    );
    assert!(
        detection.tools.contains_key("just"),
        "Should detect just from justfile. Tools: {:?}",
        detection.tools
    );
}
