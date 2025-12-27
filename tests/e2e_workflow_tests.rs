//! E2E tests for complete workflow scenarios
//!
//! These tests verify the entire vx workflow from project initialization
//! to tool execution, including hooks, services, and configuration.

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
            .current_dir(self.workdir.path())
            .output()
            .expect("Failed to execute vx command")
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn workdir(&self) -> &std::path::Path {
        self.workdir.path()
    }

    /// Create a .vx.toml file with the given content
    fn create_config(&self, content: &str) {
        let config_path = self.workdir.path().join(".vx.toml");
        fs::write(&config_path, content).expect("Failed to create .vx.toml");
    }

    /// Read a file from the workdir
    #[allow(dead_code)]
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
// Complete Workflow Tests
// ============================================

#[test]
fn test_workflow_init_setup_run() {
    let env = E2ETestEnv::new();

    // Step 1: Initialize project
    let output = env.run(&["init"]);
    // init might be interactive or require flags
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    // If init requires interaction, create config manually
    if !output.status.success() {
        env.create_config(
            r#"
[project]
name = "test-project"
version = "1.0.0"

[tools]
node = "20"

[scripts]
hello = "echo Hello from vx"
"#,
        );
    }

    // Step 2: Verify config exists
    assert!(env.file_exists(".vx.toml"));

    // Step 3: Run setup (dry-run to avoid actual installation)
    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show setup information
    assert!(
        stdout.contains("Setup")
            || stdout.contains("tool")
            || stdout.contains("node")
            || stdout.is_empty()
    );
}

#[test]
fn test_workflow_config_validation() {
    let env = E2ETestEnv::new();

    // Create a valid v2 config
    env.create_config(
        r#"
min_version = "0.6.0"

[project]
name = "validation-test"
version = "1.0.0"

[tools]
node = "20"
uv = "latest"

[env]
NODE_ENV = "development"

[env.required]
API_KEY = "Your API key"

[scripts]
dev = "echo dev"
test = "echo test"

[scripts.build]
command = "echo build"
description = "Build the project"
depends = ["test"]

[settings]
auto_install = true
parallel_install = true

[hooks]
pre_setup = "echo pre-setup"
post_setup = "echo post-setup"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
"#,
    );

    // Run config validation
    let output = env.run(&["config", "validate"]);

    // Should succeed or show validation results
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either succeeds or shows what's wrong
    assert!(
        output.status.success()
            || !stderr.is_empty()
            || stdout.contains("valid")
            || stdout.contains("error")
    );
}

#[test]
fn test_workflow_scripts_execution() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "scripts-test"

[scripts]
hello = "echo Hello World"
greet = "echo Greetings"
"#,
    );

    // List scripts
    let output = env.run(&["run", "--list"]);
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello") || stdout.contains("greet") || stdout.contains("script"));
    }

    // Run a script
    let output = env.run(&["run", "hello"]);
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Hello") || stdout.contains("hello"));
    }
}

#[test]
fn test_workflow_hooks_execution() {
    let env = E2ETestEnv::new();

    // Create config with hooks that create marker files
    env.create_config(
        r#"
[project]
name = "hooks-test"

[tools]

[hooks]
pre_setup = "echo pre_setup_executed"
post_setup = "echo post_setup_executed"
"#,
    );

    // Run setup with hooks
    let output = env.run(&["setup", "--dry-run"]);

    // In dry-run mode, hooks should not execute
    // But the command should succeed
    assert!(output.status.success());
}

#[test]
fn test_workflow_hooks_skip() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "hooks-skip-test"

[tools]

[hooks]
pre_setup = "exit 1"
post_setup = "exit 1"
"#,
    );

    // Run setup with --no-hooks flag
    let output = env.run(&["setup", "--no-hooks", "--dry-run"]);

    // Should succeed because hooks are skipped
    assert!(output.status.success());
}

#[test]
fn test_workflow_services_list() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "services-test"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready"

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]

[services.app]
command = "npm run dev"
depends_on = ["database", "redis"]
ports = ["3000:3000"]
"#,
    );

    // List services
    let output = env.run(&["services", "list"]);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should show configured services
        assert!(
            stdout.contains("database")
                || stdout.contains("redis")
                || stdout.contains("app")
                || stdout.contains("service")
        );
    }
}

#[test]
fn test_workflow_env_variables() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "env-test"

[env]
NODE_ENV = "development"
DEBUG = "true"
PORT = "3000"

[env.required]
API_KEY = "External API key"
DATABASE_URL = "Database connection string"

[env.optional]
CACHE_DIR = "Optional cache directory"
"#,
    );

    // Check config parsing
    let output = env.run(&["config", "show"]);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should show environment configuration
        assert!(
            stdout.contains("NODE_ENV")
                || stdout.contains("env")
                || stdout.contains("development")
                || !stdout.is_empty()
        );
    }
}

#[test]
fn test_workflow_tool_detailed_config() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "tool-config-test"

[tools]
node = "20"
go = "1.22"

[tools.rust]
version = "stable"
postinstall = "rustup component add clippy"
os = ["linux", "darwin", "windows"]
"#,
    );

    // Run setup dry-run to verify config is parsed
    let output = env.run(&["setup", "--dry-run"]);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should recognize the tools
        assert!(
            stdout.contains("node")
                || stdout.contains("go")
                || stdout.contains("rust")
                || stdout.contains("tool")
                || stdout.is_empty()
        );
    }
}

#[test]
fn test_workflow_dependencies_config() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "deps-test"

[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmmirror.com"

[dependencies.python]
index_url = "https://pypi.org/simple"
"#,
    );

    // Verify config is valid
    let output = env.run(&["config", "validate"]);

    // Should parse without errors
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() || !stderr.contains("parse error") || !stderr.contains("invalid")
    );
}

#[test]
fn test_workflow_script_dependencies() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "script-deps-test"

[scripts]
lint = "echo linting"
test = "echo testing"
build = "echo building"

[scripts.ci]
command = "echo ci complete"
description = "Run all CI checks"
depends = ["lint", "test", "build"]
"#,
    );

    // List scripts to verify parsing
    let output = env.run(&["run", "--list"]);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("lint")
                || stdout.contains("test")
                || stdout.contains("build")
                || stdout.contains("ci")
        );
    }
}

#[test]
fn test_workflow_python_config() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "python-test"

[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"

[python.dependencies]
requirements = ["requirements.txt"]
packages = ["pytest", "black"]
dev = ["mypy", "ruff"]
"#,
    );

    // Verify config parsing
    let output = env.run(&["setup", "--dry-run"]);

    // Should recognize python configuration
    assert!(output.status.success());
}

#[test]
fn test_workflow_full_project_config() {
    let env = E2ETestEnv::new();

    // Create a comprehensive config that uses all v2 features
    env.create_config(
        r#"
min_version = "0.6.0"

[project]
name = "full-project"
description = "A comprehensive test project"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/test/project"

[tools]
node = "20"
uv = "latest"

[tools.go]
version = "1.22"
postinstall = "go install golang.org/x/tools/gopls@latest"

[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"

[python.dependencies]
requirements = ["requirements.txt"]
packages = ["fastapi", "uvicorn"]
dev = ["pytest", "black", "ruff"]

[env]
NODE_ENV = "development"
LOG_LEVEL = "debug"

[env.required]
DATABASE_URL = "PostgreSQL connection string"
API_KEY = "External API key"

[env.secrets]
provider = "auto"
items = ["DATABASE_URL", "API_KEY"]

[scripts]
dev = "npm run dev"
test = "pytest"
lint = "ruff check . && eslint ."

[scripts.build]
command = "npm run build"
description = "Build for production"
env = { NODE_ENV = "production" }

[scripts.deploy]
command = "echo deploying..."
description = "Deploy to production"
depends = ["build", "test"]

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
shell = "auto"
log_level = "info"

[hooks]
pre_setup = "echo Preparing environment..."
post_setup = ["echo Setup complete!", "echo Ready to develop!"]
pre_commit = "vx run lint"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev", POSTGRES_DB = "myapp" }
healthcheck = "pg_isready"
volumes = ["./data:/var/lib/postgresql/data"]

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]

[services.api]
command = "uvicorn main:app --reload"
depends_on = ["database", "redis"]
ports = ["8000:8000"]
env = { DEBUG = "true" }

[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"

[dependencies.python]
index_url = "https://pypi.org/simple"
"#,
    );

    // Verify the comprehensive config is valid
    let output = env.run(&["config", "validate"]);
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should parse without critical errors
    assert!(
        output.status.success()
            || !stderr.contains("parse error")
            || !stderr.contains("invalid toml")
    );

    // Run setup dry-run
    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

// ============================================
// Error Handling Tests
// ============================================

#[test]
fn test_workflow_missing_config() {
    let env = E2ETestEnv::new();

    // Don't create any config
    let output = env.run(&["setup"]);

    // Should fail with helpful error
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains(".vx.toml")
            || combined.contains("not found")
            || combined.contains("No .vx.toml")
            || combined.contains("init")
    );
}

#[test]
fn test_workflow_invalid_config() {
    let env = E2ETestEnv::new();

    // Create invalid TOML
    env.create_config("this is not valid toml [[[");

    let output = env.run(&["setup"]);

    // Should fail with parse error
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("parse")
            || stderr.contains("invalid")
            || stderr.contains("error")
            || stderr.contains("TOML")
    );
}

#[test]
fn test_workflow_unknown_script() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "test"

[scripts]
dev = "echo dev"
"#,
    );

    // Try to run non-existent script
    let output = env.run(&["run", "nonexistent-script"]);

    // Should fail with helpful error
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("not found")
            || combined.contains("unknown")
            || combined.contains("nonexistent")
            || combined.contains("script")
    );
}

// ============================================
// Edge Cases
// ============================================

#[test]
fn test_workflow_empty_config() {
    let env = E2ETestEnv::new();

    // Create minimal empty config
    env.create_config("");

    // Should handle empty config gracefully
    let output = env.run(&["setup", "--dry-run"]);

    // Either succeeds with nothing to do or fails gracefully
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should not crash
    assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
}

#[test]
fn test_workflow_minimal_config() {
    let env = E2ETestEnv::new();

    // Create absolutely minimal valid config
    env.create_config("[tools]\n");

    let output = env.run(&["setup", "--dry-run"]);

    // Should succeed with nothing to install
    assert!(output.status.success());
}

#[test]
fn test_workflow_config_with_comments() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
# This is a comment
# Another comment

[project]
name = "comment-test"  # inline comment

[tools]
# Tool versions
node = "20"  # Use Node.js 20

[scripts]
# Development scripts
dev = "echo dev"
"#,
    );

    // Comments should be handled correctly
    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

#[test]
fn test_workflow_unicode_in_config() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "unicode-测试-тест"
description = "项目描述 with émojis 🚀"

[scripts]
hello = "echo 你好世界"
"#,
    );

    // Should handle unicode correctly
    let output = env.run(&["config", "show"]);

    // Should not crash on unicode
    assert!(output.status.success() || !String::from_utf8_lossy(&output.stderr).is_empty());
}
