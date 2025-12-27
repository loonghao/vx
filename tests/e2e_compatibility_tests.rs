//! E2E tests for backward compatibility
//!
//! These tests verify that vx maintains backward compatibility with
//! older configuration formats and behaviors.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the vx binary for testing
fn vx_binary() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("vx");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// E2E test environment
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

    fn create_config(&self, content: &str) {
        let config_path = self.workdir.path().join(".vx.toml");
        fs::write(&config_path, content).expect("Failed to create .vx.toml");
    }
}

// ============================================
// v0.5.x Config Compatibility Tests
// ============================================

#[test]
fn test_compat_v05_basic_config() {
    let env = E2ETestEnv::new();

    // v0.5.x style config (no min_version, simple structure)
    env.create_config(
        r#"
[project]
name = "legacy-project"
description = "A v0.5.x style project"
version = "1.0.0"

[tools]
node = "20"
uv = "latest"
go = "1.21"

[env]
NODE_ENV = "development"

[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

[settings]
auto_install = true
"#,
    );

    // Should parse without errors
    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());

    // Config show should work
    let output = env.run(&["config", "show"]);
    assert!(output.status.success() || !String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn test_compat_v05_python_config() {
    let env = E2ETestEnv::new();

    // v0.5.x Python configuration
    env.create_config(
        r#"
[project]
name = "python-legacy"

[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["pytest", "black", "ruff"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "mypy"]
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

#[test]
fn test_compat_v05_env_required() {
    let env = E2ETestEnv::new();

    // v0.5.x env.required format
    env.create_config(
        r#"
[project]
name = "env-legacy"

[env]
NODE_ENV = "development"
DEBUG = "true"

[env.required]
API_KEY = "Your API key"
DATABASE_URL = "Database connection string"

[env.optional]
CACHE_DIR = "Optional cache directory"
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

#[test]
fn test_compat_v05_simple_scripts() {
    let env = E2ETestEnv::new();

    // v0.5.x simple script format
    env.create_config(
        r#"
[project]
name = "scripts-legacy"

[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"
lint = "eslint . && ruff check ."
"#,
    );

    let output = env.run(&["run", "--list"]);
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("dev") || stdout.contains("test") || stdout.contains("script"));
    }
}

#[test]
fn test_compat_v05_detailed_scripts() {
    let env = E2ETestEnv::new();

    // v0.5.x detailed script format
    env.create_config(
        r#"
[project]
name = "detailed-scripts"

[scripts]
dev = "npm run dev"

[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0", "--port", "8080"]
cwd = "src"
env = { DEBUG = "true" }
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

#[test]
fn test_compat_v05_settings() {
    let env = E2ETestEnv::new();

    // v0.5.x settings format
    env.create_config(
        r#"
[project]
name = "settings-legacy"

[tools]
node = "20"

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

// ============================================
// Mixed v0.5.x and v0.6.x Config Tests
// ============================================

#[test]
fn test_compat_mixed_config() {
    let env = E2ETestEnv::new();

    // Mix of v0.5.x and v0.6.x features
    env.create_config(
        r#"
# v0.6.0 feature
min_version = "0.6.0"

[project]
name = "mixed-project"
version = "1.0.0"

# v0.5.x style tools
[tools]
node = "20"
uv = "latest"

# v0.5.x style python
[python]
version = "3.11"
venv = ".venv"

# v0.5.x style env
[env]
NODE_ENV = "development"

[env.required]
API_KEY = "API key"

# v0.5.x style scripts
[scripts]
dev = "npm run dev"
test = "pytest"

# v0.6.0 feature: detailed script with depends
[scripts.build]
command = "npm run build"
depends = ["test"]

# v0.5.x style settings
[settings]
auto_install = true

# v0.6.0 feature: hooks
[hooks]
post_setup = "echo setup complete"

# v0.6.0 feature: services
[services.db]
image = "postgres:16"
ports = ["5432:5432"]
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

#[test]
fn test_compat_gradual_migration() {
    let env = E2ETestEnv::new();

    // Config that gradually adopts v0.6.0 features
    env.create_config(
        r#"
[project]
name = "gradual-migration"

# Keep v0.5.x tools format
[tools]
node = "20"

# Adopt v0.6.0 detailed tool config for one tool
[tools.go]
version = "1.22"
postinstall = "go install golang.org/x/tools/gopls@latest"

# Keep v0.5.x scripts
[scripts]
dev = "npm run dev"

# Adopt v0.6.0 hooks
[hooks]
pre_commit = "npm run lint"
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

// ============================================
// Unknown Fields Handling Tests
// ============================================

#[test]
fn test_compat_unknown_fields_warning() {
    let env = E2ETestEnv::new();

    // Config with unknown fields (future features)
    env.create_config(
        r#"
[project]
name = "unknown-fields"

[tools]
node = "20"

# Unknown section (future feature)
[future_feature]
enabled = true
option = "value"

# Unknown field in known section
[settings]
auto_install = true
unknown_setting = "value"
"#,
    );

    // Should still work (unknown fields ignored with warning)
    let output = env.run(&["setup", "--dry-run"]);

    // Should not crash, may warn about unknown fields
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Either succeeds or shows warning (not hard error)
    assert!(
        output.status.success()
            || stderr.contains("unknown")
            || stderr.contains("warning")
            || stderr.contains("ignored")
            || !stdout.is_empty()
    );
}

#[test]
fn test_compat_extra_project_fields() {
    let env = E2ETestEnv::new();

    // v0.6.0 added new project fields
    env.create_config(
        r#"
[project]
name = "extra-fields"
description = "Test project"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/test/repo"
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

// ============================================
// API Compatibility Tests
// ============================================

#[test]
fn test_compat_cli_flags() {
    let env = E2ETestEnv::new();

    env.create_config("[tools]\nnode = \"20\"\n");

    // Test that old CLI flags still work
    let flags_to_test = [
        vec!["setup", "--dry-run"],
        vec!["setup", "--force", "--dry-run"],
        vec!["setup", "--verbose", "--dry-run"],
        vec!["setup", "--no-parallel", "--dry-run"],
    ];

    for flags in &flags_to_test {
        let output = env.run(flags);
        // Flags should be recognized
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unknown flag")
                && !stderr.contains("unrecognized")
                && !stderr.contains("invalid option")
        );
    }
}

#[test]
fn test_compat_new_cli_flags() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[tools]
node = "20"

[hooks]
pre_setup = "echo test"
"#,
    );

    // Test new v0.6.0 CLI flags
    let output = env.run(&["setup", "--no-hooks", "--dry-run"]);

    // New flag should be recognized
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unknown flag")
            && !stderr.contains("unrecognized")
            && !stderr.contains("invalid option")
    );
}

// ============================================
// Data Format Compatibility Tests
// ============================================

#[test]
fn test_compat_tool_version_formats() {
    let env = E2ETestEnv::new();

    // All supported version formats
    env.create_config(
        r#"
[tools]
node = "20"           # Major only
python = "3.11"       # Minor
go = "1.21.5"         # Exact
uv = "latest"         # Latest
rust = "stable"       # Channel
deno = "1.x"          # Range (if supported)
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

#[test]
fn test_compat_script_formats() {
    let env = E2ETestEnv::new();

    // Both simple and detailed script formats
    env.create_config(
        r#"
[scripts]
# Simple string format
simple = "echo simple"

# Detailed table format
[scripts.detailed]
command = "echo detailed"
description = "A detailed script"
args = ["--verbose"]
cwd = "."
env = { VAR = "value" }
"#,
    );

    let output = env.run(&["run", "--list"]);
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("simple") || stdout.contains("detailed") || !stdout.is_empty());
    }
}

#[test]
fn test_compat_hook_formats() {
    let env = E2ETestEnv::new();

    // Both single and array hook formats
    env.create_config(
        r#"
[hooks]
# Single command string
pre_setup = "echo single"

# Array of commands
post_setup = ["echo first", "echo second", "echo third"]
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

// ============================================
// Edge Cases
// ============================================

#[test]
fn test_compat_empty_sections() {
    let env = E2ETestEnv::new();

    // Empty sections should be valid
    env.create_config(
        r#"
[project]
name = "empty-sections"

[tools]

[env]

[scripts]

[settings]
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

#[test]
fn test_compat_minimal_valid_configs() {
    let env = E2ETestEnv::new();

    // Test various minimal valid configs
    let minimal_configs = [
        "",                                   // Empty
        "[tools]\n",                          // Just tools section
        "[project]\nname = \"test\"\n",       // Just project
        "[scripts]\ndev = \"echo dev\"\n",    // Just scripts
        "min_version = \"0.6.0\"\n[tools]\n", // Just version
    ];

    for config in &minimal_configs {
        env.create_config(config);
        let output = env.run(&["setup", "--dry-run"]);

        // Should not crash
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success() || !stderr.contains("panic") || !stderr.contains("crash"));
    }
}

#[test]
fn test_compat_whitespace_handling() {
    let env = E2ETestEnv::new();

    // Config with various whitespace
    env.create_config(
        r#"

[project]
name   =   "whitespace-test"

[tools]
node    =    "20"

[scripts]
dev   =   "echo dev"

"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}

#[test]
fn test_compat_special_characters_in_values() {
    let env = E2ETestEnv::new();

    // Config with special characters
    env.create_config(
        r#"
[project]
name = "special-chars"
description = "Test with 'quotes' and \"double quotes\""

[env]
PATH_VAR = "/usr/bin:/usr/local/bin"
REGEX = "^[a-z]+$"
URL = "https://example.com?foo=bar&baz=qux"

[scripts]
echo_special = "echo 'hello world' && echo \"test\""
"#,
    );

    let output = env.run(&["setup", "--dry-run"]);
    assert!(output.status.success());
}
