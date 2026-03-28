//! E2E tests for vx init + vx lock integration
//!
//! Tests the full chain: project detection → vx.toml generation → vx.lock resolution.
//! Uses realistic project structures modeled after popular open-source projects.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Check if a vx lock failure is caused by GitHub API rate limiting or network issues.
/// Returns true if the failure is transient (rate limit, timeout, network error),
/// meaning the test should be skipped rather than marked as failed.
fn is_transient_network_failure(stderr: &str) -> bool {
    stderr.contains("rate limit")
        || stderr.contains("403 Forbidden")
        || stderr.contains("504 Gateway Timeout")
        || stderr.contains("error decoding response body")
        || stderr.contains("timed out")
        || stderr.contains("connection refused")
        || stderr.contains("HTTP 429")
}

/// Get the path to the vx binary for testing
fn vx_binary() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    if path.ends_with("deps") {
        path.pop(); // Remove deps directory
    }
    path.push("vx");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// E2E test environment with isolated VX_HOME and working directory
struct E2ETestEnv {
    home: TempDir,
    workdir: TempDir,
}

impl E2ETestEnv {
    fn new() -> Self {
        Self {
            home: TempDir::new().expect("Failed to create temp dir for home"),
            workdir: TempDir::new().expect("Failed to create temp dir for workdir"),
        }
    }

    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(vx_binary())
            .args(args)
            .env("VX_HOME", self.home.path())
            .env("VX_PROJECT_ROOT", self.workdir.path())
            .current_dir(self.workdir.path())
            .output()
            .expect("Failed to execute vx command")
    }

    fn create_file(&self, filename: &str, content: &str) {
        let path = self.workdir.path().join(filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&path, content).expect("Failed to create file");
    }

    fn read_file(&self, name: &str) -> String {
        let path = self.workdir.path().join(name);
        fs::read_to_string(&path).unwrap_or_default()
    }

    fn file_exists(&self, name: &str) -> bool {
        self.workdir.path().join(name).exists()
    }
}

// ============================================================================
// Rust project: simulates dcc-mcp-core (the original failing case)
// Cargo.toml has rust-version = "1.90", vx lock should not fail
// ============================================================================

#[test]
fn test_init_lock_rust_project_with_msrv() {
    let env = E2ETestEnv::new();

    // Simulate dcc-mcp-core style project
    env.create_file(
        "Cargo.toml",
        r#"[package]
name = "dcc-mcp-core"
version = "0.1.0"
edition = "2021"
rust-version = "1.90"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
"#,
    );

    // Run init
    let output = env.run(&["init", "--force"]);
    assert!(
        output.status.success(),
        "vx init should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify vx.toml records the MSRV version (not "stable")
    let config = env.read_file("vx.toml");
    assert!(
        config.contains(r#"rust = "1.90"#),
        "vx.toml should contain rust = \"1.90\" (MSRV preserved). Got:\n{}",
        config
    );

    // Run lock — this was the original failing case
    let output = env.run(&["lock"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vx lock should succeed with rust MSRV version. stdout: {}\nstderr: {}",
        stdout,
        stderr
    );

    // Verify lock file was created
    assert!(env.file_exists("vx.lock"), "vx.lock should be created");
}

// ============================================================================
// Rust project with nightly channel in rust-toolchain.toml
// ============================================================================

#[test]
fn test_init_lock_rust_nightly_project() {
    let env = E2ETestEnv::new();

    env.create_file(
        "Cargo.toml",
        r#"[package]
name = "nightly-project"
version = "0.1.0"
edition = "2024"
"#,
    );
    env.create_file(
        "rust-toolchain.toml",
        r#"[toolchain]
channel = "nightly"
components = ["rustfmt", "clippy"]
"#,
    );

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    assert!(
        config.contains(r#"rust = "nightly"#),
        "vx.toml should contain rust = \"nightly\". Got:\n{}",
        config
    );

    let output = env.run(&["lock"]);
    assert!(
        output.status.success(),
        "vx lock should succeed with nightly channel. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Rust project with pinned version in rust-toolchain.toml (e.g., "1.83.0")
// ============================================================================

#[test]
fn test_init_lock_rust_pinned_toolchain() {
    let env = E2ETestEnv::new();

    env.create_file(
        "Cargo.toml",
        r#"[package]
name = "pinned-project"
version = "0.1.0"
edition = "2021"
"#,
    );
    env.create_file(
        "rust-toolchain.toml",
        r#"[toolchain]
channel = "1.83.0"
"#,
    );

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    assert!(
        config.contains(r#"rust = "1.83.0"#),
        "vx.toml should preserve exact toolchain version. Got:\n{}",
        config
    );

    let output = env.run(&["lock"]);
    assert!(
        output.status.success(),
        "vx lock should succeed with pinned rustc version 1.83.0. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Node.js project: simulates Next.js app with engines constraint
// ============================================================================

#[test]
fn test_init_lock_nextjs_style_project() {
    let env = E2ETestEnv::new();

    env.create_file(
        "package.json",
        r#"{
  "name": "my-nextjs-app",
  "version": "0.1.0",
  "engines": {
    "node": ">=20.0.0"
  },
  "dependencies": {
    "next": "14.0.0",
    "react": "18.2.0",
    "react-dom": "18.2.0"
  },
  "devDependencies": {
    "typescript": "5.3.0"
  }
}"#,
    );
    env.create_file("pnpm-lock.yaml", "lockfileVersion: '9.0'\n");

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    // Should detect node with major version from engines
    assert!(
        config.contains("node = \"20\"") || config.contains("node ="),
        "vx.toml should detect node from engines. Got:\n{}",
        config
    );
    // Should detect pnpm from lockfile
    assert!(
        config.contains("pnpm"),
        "vx.toml should detect pnpm from pnpm-lock.yaml. Got:\n{}",
        config
    );

    let output = env.run(&["lock"]);
    assert!(
        output.status.success(),
        "vx lock should succeed for Next.js project. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Python project: simulates FastAPI project with pyproject.toml
// ============================================================================

#[test]
fn test_init_lock_fastapi_style_project() {
    let env = E2ETestEnv::new();

    env.create_file(
        "pyproject.toml",
        r#"[project]
name = "my-fastapi-app"
version = "0.1.0"
requires-python = ">=3.11"
dependencies = [
    "fastapi>=0.100.0",
    "uvicorn>=0.20.0",
]

[tool.uv]
dev-dependencies = ["pytest", "httpx"]
"#,
    );
    env.create_file("uv.lock", "version = 1\n");

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    assert!(
        config.contains("uv"),
        "vx.toml should detect uv. Got:\n{}",
        config
    );
    assert!(
        config.contains("python = \"3.11\""),
        "vx.toml should detect python 3.11 from requires-python. Got:\n{}",
        config
    );

    let output = env.run(&["lock"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() && is_transient_network_failure(&stderr) {
        eprintln!("SKIPPED: transient network failure (rate limit / timeout): {stderr}");
        return;
    }
    assert!(
        output.status.success(),
        "vx lock should succeed for FastAPI project. stderr: {}",
        stderr
    );
}

// ============================================================================
// Go project: simulates a typical Go module
// ============================================================================

#[test]
fn test_init_lock_go_module_project() {
    let env = E2ETestEnv::new();

    env.create_file(
        "go.mod",
        "module github.com/user/my-go-service\n\ngo 1.22\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.1\n)\n",
    );

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    assert!(
        config.contains("go"),
        "vx.toml should detect go. Got:\n{}",
        config
    );

    let output = env.run(&["lock"]);
    assert!(
        output.status.success(),
        "vx lock should succeed for Go project. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Mixed project: Rust + Python (PyO3/maturin style, like cryptography)
// ============================================================================

#[test]
fn test_init_lock_pyo3_mixed_project() {
    let env = E2ETestEnv::new();

    env.create_file(
        "Cargo.toml",
        r#"[package]
name = "my-pyo3-lib"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[lib]
name = "my_pyo3_lib"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }
"#,
    );
    env.create_file(
        "pyproject.toml",
        r#"[project]
name = "my-pyo3-lib"
version = "0.1.0"
requires-python = ">=3.9"

[build-system]
requires = ["maturin>=1.0"]
build-backend = "maturin"
"#,
    );

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    // Should detect both rust (with MSRV) and maturin
    assert!(
        config.contains(r#"rust = "1.75"#),
        "vx.toml should preserve rust MSRV 1.75. Got:\n{}",
        config
    );
    assert!(
        config.contains("maturin"),
        "vx.toml should detect maturin. Got:\n{}",
        config
    );

    let output = env.run(&["lock"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() && is_transient_network_failure(&stderr) {
        eprintln!("SKIPPED: transient network failure (rate limit / timeout): {stderr}");
        return;
    }
    assert!(
        output.status.success(),
        "vx lock should succeed for PyO3/maturin project. stderr: {}",
        stderr
    );
}

// ============================================================================
// Monorepo: Node.js + Go + Python (fullstack project)
// ============================================================================

#[test]
fn test_init_lock_monorepo_fullstack() {
    let env = E2ETestEnv::new();

    // Frontend
    env.create_file(
        "package.json",
        r#"{
  "name": "fullstack-monorepo",
  "engines": { "node": ">=22" },
  "packageManager": "pnpm@9.0.0"
}"#,
    );
    env.create_file("pnpm-lock.yaml", "lockfileVersion: '9.0'\n");

    // Backend Go service
    env.create_file("go.mod", "module github.com/user/fullstack\n\ngo 1.22\n");

    // Python scripts/tools
    env.create_file(
        "pyproject.toml",
        "[project]\nname = \"scripts\"\nrequires-python = \">=3.12\"\n",
    );

    // Justfile for task orchestration
    env.create_file("justfile", "default:\n  echo \"fullstack monorepo\"\n");

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    // Should detect all ecosystems
    assert!(config.contains("node"), "Should detect node");
    assert!(config.contains("go"), "Should detect go");
    assert!(
        config.contains("python") || config.contains("uv"),
        "Should detect python/uv"
    );
    assert!(config.contains("just"), "Should detect just");
    assert!(config.contains("pnpm"), "Should detect pnpm");

    let output = env.run(&["lock"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() && is_transient_network_failure(&stderr) {
        eprintln!("SKIPPED: transient network failure (rate limit / timeout): {stderr}");
        return;
    }
    assert!(
        output.status.success(),
        "vx lock should succeed for monorepo. stderr: {}",
        stderr
    );
}

// ============================================================================
// Manual vx.toml with various version formats → vx lock
// ============================================================================

#[test]
fn test_lock_manual_vx_toml_rust_stable() {
    let env = E2ETestEnv::new();

    env.create_file(
        "vx.toml",
        r#"[tools]
rust = "stable"
"#,
    );

    let output = env.run(&["lock"]);
    assert!(
        output.status.success(),
        "vx lock should accept rust = \"stable\". stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_lock_manual_vx_toml_rust_beta() {
    let env = E2ETestEnv::new();

    env.create_file(
        "vx.toml",
        r#"[tools]
rust = "beta"
"#,
    );

    let output = env.run(&["lock"]);
    assert!(
        output.status.success(),
        "vx lock should accept rust = \"beta\". stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_lock_manual_vx_toml_rust_numeric_version() {
    let env = E2ETestEnv::new();

    // The exact version that caused the original bug
    env.create_file(
        "vx.toml",
        r#"[tools]
rust = "1.90"
"#,
    );

    let output = env.run(&["lock"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vx lock should NOT fail with 'No version found for rust matching 1.90'. stderr: {}",
        stderr
    );
    // Should NOT contain the original error message
    assert!(
        !stderr.contains("No version found for rust matching 1.90"),
        "Should not get version resolution error for rust 1.90"
    );
}

#[test]
fn test_lock_manual_vx_toml_multiple_tools() {
    let env = E2ETestEnv::new();

    env.create_file(
        "vx.toml",
        r#"[tools]
node = "22"
uv = "latest"
just = "latest"
"#,
    );

    let output = env.run(&["lock"]);
    assert!(
        output.status.success(),
        "vx lock should resolve multiple tools. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let lock = env.read_file("vx.lock");
    assert!(
        lock.contains("node") && lock.contains("uv") && lock.contains("just"),
        "Lock file should contain all tools. Got:\n{}",
        lock
    );
}

// ============================================================================
// vx init --force should preserve user-specified versions
// ============================================================================

#[test]
fn test_init_force_preserves_user_rust_version() {
    let env = E2ETestEnv::new();

    // User manually set rust version
    env.create_file(
        "vx.toml",
        r#"[tools]
rust = "1.85.0"
"#,
    );

    // Project has different MSRV
    env.create_file(
        "Cargo.toml",
        r#"[package]
name = "test-proj"
version = "0.1.0"
rust-version = "1.70.0"
"#,
    );

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    // The user's manually set version (1.85.0) should be preserved
    // over the project-detected version (1.70.0)
    assert!(
        config.contains(r#"rust = "1.85.0"#),
        "init --force should preserve user-specified tool version. Got:\n{}",
        config
    );
}

// ============================================================================
// .NET project with global.json SDK version
// ============================================================================

#[test]
#[ignore = "dotnet provider requires network and may not have versions available"]
fn test_init_lock_dotnet_project() {
    let env = E2ETestEnv::new();

    env.create_file(
        "MyApp.csproj",
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
    );
    env.create_file("global.json", r#"{ "sdk": { "version": "8.0.100" } }"#);

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    assert!(
        config.contains("dotnet"),
        "vx.toml should detect dotnet. Got:\n{}",
        config
    );

    let output = env.run(&["lock"]);
    assert!(
        output.status.success(),
        "vx lock should succeed for .NET project. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Node.js project with yarn (packageManager field)
// ============================================================================

#[test]
fn test_init_lock_yarn_project() {
    let env = E2ETestEnv::new();

    env.create_file(
        "package.json",
        r#"{
  "name": "yarn-app",
  "packageManager": "yarn@4.0.0",
  "engines": { "node": ">=18" }
}"#,
    );
    env.create_file("yarn.lock", "# yarn lockfile v1\n");

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    assert!(
        config.contains("yarn"),
        "vx.toml should detect yarn. Got:\n{}",
        config
    );

    let output = env.run(&["lock"]);
    assert!(
        output.status.success(),
        "vx lock should succeed for yarn project. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Bun project
// ============================================================================

#[test]
fn test_init_lock_bun_project() {
    let env = E2ETestEnv::new();

    env.create_file(
        "package.json",
        r#"{
  "name": "bun-app",
  "packageManager": "bun@1.1.0"
}"#,
    );
    env.create_file("bun.lockb", "binary-data");

    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let config = env.read_file("vx.toml");
    assert!(
        config.contains("bun"),
        "vx.toml should detect bun. Got:\n{}",
        config
    );

    let output = env.run(&["lock"]);
    assert!(
        output.status.success(),
        "vx lock should succeed for bun project. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Lock with --dry-run should not write file
// ============================================================================

#[test]
fn test_lock_dry_run_no_file() {
    let env = E2ETestEnv::new();

    env.create_file("vx.toml", "[tools]\njust = \"latest\"\n");

    let output = env.run(&["lock", "--dry-run"]);
    assert!(output.status.success());

    assert!(
        !env.file_exists("vx.lock"),
        "vx.lock should NOT be created in dry-run mode"
    );
}
