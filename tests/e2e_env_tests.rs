//! E2E tests for the env command
//!
//! These tests verify the environment management functionality.
//!
//! Note: These tests use `--global` flag to create environments in VX_HOME,
//! which is isolated via tempdir. Project-local environments would require
//! changing the working directory, which is more complex.

use std::env;
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
            .current_dir(self.workdir.path())
            .output()
            .expect("Failed to execute vx command")
    }

    fn run_success(&self, args: &[&str]) -> String {
        let output = self.run(args);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            panic!(
                "Command failed: vx {}\nstdout: {}\nstderr: {}",
                args.join(" "),
                stdout,
                stderr
            );
        }
        stdout
    }

    /// Create a .vx.toml file in the workdir to enable project environment commands
    fn create_vx_config(&self) {
        let config_path = self.workdir.path().join(".vx.toml");
        std::fs::write(&config_path, "[tools]\n").expect("Failed to create .vx.toml");
    }
}

#[test]
fn test_env_list_empty() {
    let env = E2ETestEnv::new();
    let output = env.run(&["env", "list"]);
    assert!(output.status.success());

    // With a fresh VX_HOME, should show "Environments:" header or be empty
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The list command should succeed and produce some output
    // (even if just "Environments:" with nothing listed)
    assert!(
        stdout.contains("Environments") || stdout.contains("No environments") || stdout.is_empty()
    );
}

#[test]
fn test_env_create() {
    let env = E2ETestEnv::new();
    env.create_vx_config();

    // Create a project environment (requires .vx.toml)
    let output = env.run(&["env", "create"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Created") || stdout.contains("project"));
}

#[test]
fn test_env_create_global() {
    let env = E2ETestEnv::new();

    // Create a global environment
    let output = env.run(&["env", "create", "--global", "test-env"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Created") || stdout.contains("test-env"));
}

#[test]
fn test_env_create_and_list() {
    let env = E2ETestEnv::new();

    // Create global environment
    let _ = env.run_success(&["env", "create", "--global", "my-project"]);

    // List environments
    let output = env.run(&["env", "list"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("my-project"));
}

#[test]
fn test_env_show() {
    let env = E2ETestEnv::new();

    // Create global environment first
    let _ = env.run_success(&["env", "create", "--global", "show-test"]);

    // Show environment details
    let output = env.run(&["env", "show", "show-test"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("show-test") || !stdout.is_empty());
}

#[test]
fn test_env_delete() {
    let env = E2ETestEnv::new();

    // Create global environment
    let _ = env.run_success(&["env", "create", "--global", "to-delete"]);

    // Delete environment
    let output = env.run(&["env", "delete", "--global", "to-delete", "--force"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Deleted") || stdout.contains("to-delete"));
}

#[test]
fn test_env_use() {
    let env = E2ETestEnv::new();

    // Create global environment
    let _ = env.run_success(&["env", "create", "--global", "use-test"]);

    // Use/activate environment
    let output = env.run(&["env", "use", "--global", "use-test"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Activated") || stdout.contains("use-test") || stdout.contains("activate")
    );
}

#[test]
fn test_env_create_duplicate() {
    let env = E2ETestEnv::new();

    // Create global environment
    let _ = env.run_success(&["env", "create", "--global", "duplicate-test"]);

    // Try to create same environment again
    let output = env.run(&["env", "create", "--global", "duplicate-test"]);

    // Should fail or warn about existing environment
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Either fails or warns about existing
    assert!(
        !output.status.success()
            || combined.contains("already exists")
            || combined.contains("exists")
    );
}

#[test]
fn test_env_delete_nonexistent() {
    let env = E2ETestEnv::new();

    // Try to delete non-existent global environment
    let output = env.run(&["env", "delete", "--global", "nonexistent-env", "--force"]);

    // Should fail with error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Either fails or shows not found message
    assert!(
        !output.status.success()
            || stderr.contains("not found")
            || stdout.contains("not found")
            || stderr.contains("does not exist")
    );
}

#[test]
fn test_env_help() {
    let env = E2ETestEnv::new();

    let output = env.run(&["env", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("create") || stdout.contains("list") || stdout.contains("Environment"));
}
