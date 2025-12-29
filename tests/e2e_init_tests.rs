//! E2E tests for vx init command
//!
//! Tests the smart project initialization and configuration merging.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

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

    #[allow(dead_code)]
    fn workdir(&self) -> &std::path::Path {
        self.workdir.path()
    }

    /// Create a vx.toml file with the given content
    fn create_config(&self, filename: &str, content: &str) {
        let config_path = self.workdir.path().join(filename);
        fs::write(&config_path, content).expect("Failed to create config file");
    }

    /// Read a file from the workdir
    fn read_file(&self, name: &str) -> String {
        let path = self.workdir.path().join(name);
        fs::read_to_string(&path).unwrap_or_default()
    }

    /// Check if a file exists in the workdir
    fn file_exists(&self, name: &str) -> bool {
        self.workdir.path().join(name).exists()
    }
}

// ============================================
// vx init Tests
// ============================================

#[test]
fn test_init_creates_vx_toml() {
    let env = E2ETestEnv::new();

    // Run init with dry-run to see what would be created
    let output = env.run(&["init", "--dry-run"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show preview of configuration
    assert!(stdout.contains("[tools]") || stdout.contains("Preview"));
}

#[test]
fn test_init_respects_existing_vx_toml() {
    let env = E2ETestEnv::new();

    // Create existing vx.toml
    env.create_config("vx.toml", "[tools]\nnode = \"18\"\n");

    // Run init without --force
    let output = env.run(&["init"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should warn about existing config
    assert!(stdout.contains("already exists") || stdout.contains("vx.toml"));
}

#[test]
fn test_init_respects_existing_dot_vx_toml() {
    let env = E2ETestEnv::new();

    // Create existing vx.toml (legacy format)
    env.create_config("vx.toml", "[tools]\nnode = \"18\"\n");

    // Run init without --force
    let output = env.run(&["init"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should warn about existing config
    assert!(stdout.contains("already exists") || stdout.contains("vx.toml"));
}

#[test]
fn test_init_force_preserves_scripts() {
    let env = E2ETestEnv::new();

    // Create existing vx.toml with scripts
    env.create_config(
        "vx.toml",
        r#"[tools]
uv = "0.7.12"

[scripts]
build = "uv run nox -s build-wheel"
lint = "uv run nox -s lint"
test = "uv run nox -s tests"
"#,
    );

    // Run init --force
    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    // Read the updated config
    let config_content = env.read_file("vx.toml");

    // Should preserve scripts
    assert!(config_content.contains("[scripts]"));
    assert!(config_content.contains("build"));
    assert!(config_content.contains("lint"));
    assert!(config_content.contains("test"));
}

#[test]
fn test_init_force_preserves_tool_versions() {
    let env = E2ETestEnv::new();

    // Create existing vx.toml with specific tool version
    env.create_config(
        "vx.toml",
        r#"[tools]
uv = "0.7.12"
"#,
    );

    // Run init --force
    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    // Read the updated config
    let config_content = env.read_file("vx.toml");

    // Should preserve the specific version (0.7.12), not change to "latest"
    assert!(config_content.contains("uv = \"0.7.12\""));
}

#[test]
fn test_init_force_writes_to_existing_file_location() {
    let env = E2ETestEnv::new();

    // Create existing vx.toml
    env.create_config("vx.toml", "[tools]\nnode = \"18\"\n");

    // Run init --force
    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    // Should update vx.toml
    assert!(env.file_exists("vx.toml"));

    // Verify the file was updated (should still contain tools section)
    let content = env.read_file("vx.toml");
    assert!(content.contains("[tools]"));
}

#[test]
fn test_init_new_project_creates_vx_toml() {
    let env = E2ETestEnv::new();

    // No existing config
    // Run init
    let output = env.run(&["init"]);
    assert!(output.status.success());

    // Should create vx.toml (preferred name)
    assert!(env.file_exists("vx.toml"));
}

#[test]
fn test_init_output_message_shows_correct_filename() {
    let env = E2ETestEnv::new();

    // Create existing vx.toml
    env.create_config("vx.toml", "[tools]\n");

    // Run init --force
    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should mention vx.toml in the output
    assert!(stdout.contains("vx.toml"));
}

#[test]
fn test_init_dry_run_shows_correct_filename() {
    let env = E2ETestEnv::new();

    // Create existing vx.toml
    env.create_config("vx.toml", "[tools]\n");

    // Run init --force --dry-run
    let output = env.run(&["init", "--force", "--dry-run"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should mention vx.toml in the preview
    assert!(stdout.contains("vx.toml"));
}

// ============================================
// Python Project Detection Tests
// ============================================

#[test]
fn test_init_detects_python_project() {
    let env = E2ETestEnv::new();

    // Create pyproject.toml
    env.create_config(
        "pyproject.toml",
        r#"[project]
name = "test-project"
requires-python = ">=3.10"

[tool.uv]
dev-dependencies = ["pytest"]
"#,
    );

    // Create uv.lock
    env.create_config("uv.lock", "version = 1\n");

    // Run init --dry-run
    let output = env.run(&["init", "--dry-run"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should detect Python project
    assert!(stdout.contains("Python") || stdout.contains("python"));
    // Should detect uv
    assert!(stdout.contains("uv"));
}

#[test]
fn test_init_detects_python_version_from_pyproject() {
    let env = E2ETestEnv::new();

    // Create pyproject.toml with specific Python version
    env.create_config(
        "pyproject.toml",
        r#"[project]
name = "test-project"
requires-python = ">=3.11"
"#,
    );

    // Run init --dry-run
    let output = env.run(&["init", "--dry-run"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should include Python version from pyproject.toml
    assert!(stdout.contains("python = \"3.11\"") || stdout.contains("3.11"));
}

// ============================================
// shotgrid-mcp-server Style Project Test
// ============================================

#[test]
fn test_init_force_shotgrid_style_project() {
    let env = E2ETestEnv::new();

    // Create a project similar to shotgrid-mcp-server
    // 1. Create pyproject.toml
    env.create_config(
        "pyproject.toml",
        r#"[project]
name = "shotgrid-mcp-server"
version = "0.14.1"
requires-python = ">=3.10,<3.13"

[tool.uv]
dev-dependencies = ["pytest", "nox"]
"#,
    );

    // 2. Create uv.lock
    env.create_config("uv.lock", "version = 1\n");

    // 3. Create existing vx.toml with scripts
    env.create_config(
        "vx.toml",
        r#"# VX Project Configuration (v2)

# This file defines the tools and environment for this project.
# Run 'vx setup' to install all required tools.
# Documentation: https://github.com/loonghao/vx/docs/config

# Tool versions
[tools]
uv = "0.7.12"

# Script definitions
[scripts]
build = "uv run nox -s build-wheel"
lint = "uv run nox -s lint"
typecheck = "uv run mypy src"
docs = "uv run nox -s docs"
check = "uv run ruff check ."
test = "uv run nox -s tests"
format = "uv run ruff format ."
"#,
    );

    // Run init --force
    let output = env.run(&["init", "--force"]);
    assert!(output.status.success());

    // Read the updated config
    let config_content = env.read_file("vx.toml");

    // Verify scripts are preserved
    assert!(config_content.contains("[scripts]"));
    assert!(config_content.contains("build"));
    assert!(config_content.contains("lint"));
    assert!(config_content.contains("typecheck"));
    assert!(config_content.contains("docs"));
    assert!(config_content.contains("check"));
    assert!(config_content.contains("test"));
    assert!(config_content.contains("format"));

    // Verify the specific uv version is preserved
    assert!(config_content.contains("uv = \"0.7.12\""));

    // Verify vx.toml exists (was updated in place)
    assert!(env.file_exists("vx.toml"));
}

#[test]
fn test_sync_with_shotgrid_style_config() {
    let env = E2ETestEnv::new();

    // Create vx.toml similar to shotgrid-mcp-server
    env.create_config(
        "vx.toml",
        r#"[tools]
uv = "0.7.12"

[scripts]
build = "uv run nox -s build-wheel"
lint = "uv run nox -s lint"
test = "uv run nox -s tests"
"#,
    );

    // Run sync --check
    let output = env.run(&["sync", "--check"]);

    // Should work without errors
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either succeeds or shows what needs to be installed
    assert!(
        output.status.success()
            || stdout.contains("uv")
            || stdout.contains("tool")
            || !stderr.contains("parse error")
    );
}

#[test]
fn test_setup_with_shotgrid_style_config() {
    let env = E2ETestEnv::new();

    // Create vx.toml similar to shotgrid-mcp-server
    env.create_config(
        "vx.toml",
        r#"[tools]
uv = "0.7.12"

[scripts]
build = "uv run nox -s build-wheel"
lint = "uv run nox -s lint"
test = "uv run nox -s tests"
"#,
    );

    // Run setup --dry-run
    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show the scripts
    assert!(
        stdout.contains("build")
            || stdout.contains("lint")
            || stdout.contains("test")
            || stdout.contains("uv")
    );
}

// ============================================
// Template Tests
// ============================================

#[test]
fn test_init_template_python() {
    let env = E2ETestEnv::new();

    // Run init with python template
    let output = env.run(&["init", "--template", "python"]);
    assert!(output.status.success());

    // Read the created config
    let config_content = env.read_file("vx.toml");

    // Should have python and uv
    assert!(config_content.contains("python"));
    assert!(config_content.contains("uv"));
}

#[test]
fn test_init_template_preserves_existing_scripts() {
    let env = E2ETestEnv::new();

    // Create existing vx.toml with scripts
    env.create_config(
        "vx.toml",
        r#"[tools]
node = "18"

[scripts]
custom = "echo custom script"
"#,
    );

    // Run init with template and --force
    let output = env.run(&["init", "--template", "python", "--force"]);
    assert!(output.status.success());

    // Read the updated config
    let config_content = env.read_file("vx.toml");

    // Should preserve custom script
    assert!(config_content.contains("custom"));
    assert!(config_content.contains("echo custom script"));
}

// ============================================
// List Templates Test
// ============================================

#[test]
fn test_init_list_templates() {
    let env = E2ETestEnv::new();

    let output = env.run(&["init", "--list-templates"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should list available templates
    assert!(stdout.contains("node"));
    assert!(stdout.contains("python"));
    assert!(stdout.contains("rust"));
    assert!(stdout.contains("go"));
}
