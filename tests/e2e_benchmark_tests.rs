//! E2E performance benchmark tests
//!
//! These tests measure the performance of vx operations to ensure
//! they meet acceptable thresholds.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
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

    fn run_timed(&self, args: &[&str]) -> (std::process::Output, Duration) {
        let start = Instant::now();
        let output = self.run(args);
        let duration = start.elapsed();
        (output, duration)
    }

    fn create_config(&self, content: &str) {
        let config_path = self.workdir.path().join("vx.toml");
        fs::write(&config_path, content).expect("Failed to create vx.toml");
    }
}

/// Performance thresholds (in milliseconds)
/// Note: Windows CI is typically slower than Linux, so we use higher thresholds on Windows
/// Note: Debug builds are significantly slower than release builds
mod thresholds {
    /// Maximum time for CLI startup (no operation)
    pub const CLI_STARTUP_MS: u64 = 3000;

    /// Maximum time for help command
    /// Note: Increased to 500ms to account for debug builds and system load variability
    #[cfg(windows)]
    pub const HELP_MS: u64 = 500;
    #[cfg(not(windows))]
    pub const HELP_MS: u64 = 350;

    /// Maximum time for version command
    /// Note: Increased to 500ms to account for debug builds and system load variability
    #[cfg(windows)]
    pub const VERSION_MS: u64 = 500;
    #[cfg(not(windows))]
    pub const VERSION_MS: u64 = 350;

    /// Maximum time for config parsing (small config)
    /// Note: Increased to 1500ms to account for Windows CI variability
    #[cfg(windows)]
    pub const CONFIG_PARSE_SMALL_MS: u64 = 1500;
    #[cfg(not(windows))]
    pub const CONFIG_PARSE_SMALL_MS: u64 = 1000;

    /// Maximum time for config parsing (large config)
    pub const CONFIG_PARSE_LARGE_MS: u64 = 3000;

    /// Maximum time for setup dry-run (small config)
    /// Note: macOS CI runners have higher variability, so we use a more generous threshold
    #[cfg(target_os = "macos")]
    pub const SETUP_DRYRUN_SMALL_MS: u64 = 1500;
    #[cfg(not(target_os = "macos"))]
    pub const SETUP_DRYRUN_SMALL_MS: u64 = 1000;

    /// Maximum time for setup dry-run (large config)
    #[cfg(target_os = "macos")]
    pub const SETUP_DRYRUN_LARGE_MS: u64 = 4000;
    #[cfg(not(target_os = "macos"))]
    pub const SETUP_DRYRUN_LARGE_MS: u64 = 3000;

    /// Maximum time for script listing
    pub const SCRIPT_LIST_MS: u64 = 1000;

    /// Maximum time for config validation
    /// Note: Increased to 1500ms to account for Windows CI variability
    #[cfg(windows)]
    pub const CONFIG_VALIDATE_MS: u64 = 1500;
    #[cfg(not(windows))]
    pub const CONFIG_VALIDATE_MS: u64 = 1000;
}

// ============================================
// CLI Startup Performance Tests
// ============================================

#[test]
fn bench_cli_help() {
    let env = E2ETestEnv::new();

    // Warm up
    let _ = env.run(&["--help"]);

    // Measure
    let (output, duration) = env.run_timed(&["--help"]);

    assert!(output.status.success());
    assert!(
        duration.as_millis() < thresholds::HELP_MS as u128,
        "Help command took {}ms, expected < {}ms",
        duration.as_millis(),
        thresholds::HELP_MS
    );

    println!("bench_cli_help: {}ms", duration.as_millis());
}

#[test]
fn bench_cli_version() {
    let env = E2ETestEnv::new();

    // Warm up
    let _ = env.run(&["--version"]);

    // Measure
    let (output, duration) = env.run_timed(&["--version"]);

    assert!(output.status.success());
    assert!(
        duration.as_millis() < thresholds::VERSION_MS as u128,
        "Version command took {}ms, expected < {}ms",
        duration.as_millis(),
        thresholds::VERSION_MS
    );

    println!("bench_cli_version: {}ms", duration.as_millis());
}

#[test]
fn bench_cli_startup() {
    let env = E2ETestEnv::new();

    // Measure startup with no valid command (will fail but measures startup time)
    let start = Instant::now();
    let _ = env.run(&["__nonexistent__"]);
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < thresholds::CLI_STARTUP_MS as u128,
        "CLI startup took {}ms, expected < {}ms",
        duration.as_millis(),
        thresholds::CLI_STARTUP_MS
    );

    println!("bench_cli_startup: {}ms", duration.as_millis());
}

// ============================================
// Config Parsing Performance Tests
// ============================================

#[test]
fn bench_config_parse_small() {
    let env = E2ETestEnv::new();

    // Small config
    env.create_config(
        r#"
[project]
name = "small-config"

[tools]
node = "20"
uv = "latest"

[scripts]
dev = "npm run dev"
test = "pytest"
"#,
    );

    // Warm up
    let _ = env.run(&["config", "show"]);

    // Measure
    let (output, duration) = env.run_timed(&["config", "show"]);

    assert!(output.status.success() || !String::from_utf8_lossy(&output.stderr).is_empty());
    assert!(
        duration.as_millis() < thresholds::CONFIG_PARSE_SMALL_MS as u128,
        "Small config parse took {}ms, expected < {}ms",
        duration.as_millis(),
        thresholds::CONFIG_PARSE_SMALL_MS
    );

    println!("bench_config_parse_small: {}ms", duration.as_millis());
}

#[test]
fn bench_config_parse_large() {
    let env = E2ETestEnv::new();

    // Large comprehensive config
    env.create_config(
        r#"
min_version = "0.6.0"

[project]
name = "large-config"
description = "A comprehensive test project with many settings"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/test/project"

[tools]
node = "20"
uv = "latest"
go = "1.22"
rust = "stable"
deno = "1.40"
bun = "latest"

[tools.python]
version = "3.12"
postinstall = "pip install --upgrade pip"

[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"

[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt", "requirements-test.txt"]
packages = ["fastapi", "uvicorn", "sqlalchemy", "alembic", "pydantic"]
dev = ["pytest", "black", "ruff", "mypy", "isort", "pre-commit"]

[env]
NODE_ENV = "development"
PYTHON_ENV = "development"
LOG_LEVEL = "debug"
DEBUG = "true"
PORT = "3000"
API_PORT = "8000"

[env.required]
DATABASE_URL = "PostgreSQL connection string"
REDIS_URL = "Redis connection string"
API_KEY = "External API key"
JWT_SECRET = "JWT signing secret"
AWS_ACCESS_KEY_ID = "AWS access key"
AWS_SECRET_ACCESS_KEY = "AWS secret key"

[env.optional]
SENTRY_DSN = "Sentry error tracking DSN"
CACHE_TTL = "Cache time-to-live in seconds"
MAX_CONNECTIONS = "Maximum database connections"

[env.secrets]
provider = "auto"
items = ["DATABASE_URL", "REDIS_URL", "API_KEY", "JWT_SECRET"]

[scripts]
dev = "npm run dev"
test = "pytest"
lint = "ruff check . && eslint ."
format = "black . && prettier --write ."
typecheck = "mypy . && tsc --noEmit"

[scripts.build]
command = "npm run build"
description = "Build for production"
env = { NODE_ENV = "production" }
depends = ["lint", "typecheck", "test"]

[scripts.deploy]
command = "npm run deploy"
description = "Deploy to production"
depends = ["build"]

[scripts.db:migrate]
command = "alembic upgrade head"
description = "Run database migrations"

[scripts.db:rollback]
command = "alembic downgrade -1"
description = "Rollback last migration"

[scripts.db:seed]
command = "python scripts/seed.py"
description = "Seed database with test data"

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
shell = "auto"
log_level = "info"

[settings.experimental]
monorepo = false
workspaces = false

[hooks]
pre_setup = "echo Preparing environment..."
post_setup = ["npm install", "pip install -r requirements.txt", "vx run db:migrate"]
pre_commit = "vx run lint && vx run typecheck"
enter = "vx sync --check"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev", POSTGRES_DB = "myapp", POSTGRES_USER = "dev" }
healthcheck = "pg_isready -U dev"
volumes = ["./data/postgres:/var/lib/postgresql/data"]

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]
healthcheck = "redis-cli ping"

[services.elasticsearch]
image = "elasticsearch:8.11.0"
ports = ["9200:9200", "9300:9300"]
env = { "discovery.type" = "single-node", "xpack.security.enabled" = "false" }

[services.api]
command = "uvicorn main:app --reload --host 0.0.0.0 --port 8000"
depends_on = ["database", "redis"]
ports = ["8000:8000"]
env = { DEBUG = "true" }

[services.worker]
command = "celery -A tasks worker --loglevel=info"
depends_on = ["database", "redis"]

[services.frontend]
command = "npm run dev"
depends_on = ["api"]
ports = ["3000:3000"]

[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmjs.org"

[dependencies.python]
index_url = "https://pypi.org/simple"
extra_index_urls = ["https://pypi.example.com/simple"]

[dependencies.constraints]
"lodash" = ">=4.17.21"
"requests" = ">=2.31.0"
"*" = { licenses = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC", "MPL-2.0"] }
"#,
    );

    // Warm up
    let _ = env.run(&["config", "show"]);

    // Measure
    let (output, duration) = env.run_timed(&["config", "show"]);

    assert!(output.status.success() || !String::from_utf8_lossy(&output.stderr).is_empty());
    assert!(
        duration.as_millis() < thresholds::CONFIG_PARSE_LARGE_MS as u128,
        "Large config parse took {}ms, expected < {}ms",
        duration.as_millis(),
        thresholds::CONFIG_PARSE_LARGE_MS
    );

    println!("bench_config_parse_large: {}ms", duration.as_millis());
}

// ============================================
// Setup Performance Tests
// ============================================

#[test]
fn bench_setup_dryrun_small() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "small-setup"

[tools]
node = "20"
"#,
    );

    // Warm up
    let _ = env.run(&["setup", "--dry-run"]);

    // Measure
    let (output, duration) = env.run_timed(&["setup", "--dry-run"]);

    assert!(output.status.success());
    assert!(
        duration.as_millis() < thresholds::SETUP_DRYRUN_SMALL_MS as u128,
        "Small setup dry-run took {}ms, expected < {}ms",
        duration.as_millis(),
        thresholds::SETUP_DRYRUN_SMALL_MS
    );

    println!("bench_setup_dryrun_small: {}ms", duration.as_millis());
}

#[test]
fn bench_setup_dryrun_large() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
min_version = "0.6.0"

[project]
name = "large-setup"

[tools]
node = "20"
uv = "latest"
go = "1.22"
rust = "stable"
deno = "1.40"
bun = "latest"

[python]
version = "3.12"
venv = ".venv"

[hooks]
pre_setup = "echo pre"
post_setup = ["echo post1", "echo post2"]

[services.db]
image = "postgres:16"
ports = ["5432:5432"]

[services.redis]
image = "redis:7"
ports = ["6379:6379"]
"#,
    );

    // Warm up
    let _ = env.run(&["setup", "--dry-run"]);

    // Measure
    let (output, duration) = env.run_timed(&["setup", "--dry-run"]);

    assert!(output.status.success());
    assert!(
        duration.as_millis() < thresholds::SETUP_DRYRUN_LARGE_MS as u128,
        "Large setup dry-run took {}ms, expected < {}ms",
        duration.as_millis(),
        thresholds::SETUP_DRYRUN_LARGE_MS
    );

    println!("bench_setup_dryrun_large: {}ms", duration.as_millis());
}

// ============================================
// Script Performance Tests
// ============================================

#[test]
fn bench_script_list() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "scripts-bench"

[scripts]
dev = "npm run dev"
test = "pytest"
lint = "eslint ."
format = "prettier --write ."
build = "npm run build"
deploy = "npm run deploy"
clean = "rm -rf dist"
docs = "npm run docs"
"#,
    );

    // Warm up
    let _ = env.run(&["run", "--list"]);

    // Measure
    let (output, duration) = env.run_timed(&["run", "--list"]);

    assert!(output.status.success() || !String::from_utf8_lossy(&output.stderr).is_empty());
    assert!(
        duration.as_millis() < thresholds::SCRIPT_LIST_MS as u128,
        "Script list took {}ms, expected < {}ms",
        duration.as_millis(),
        thresholds::SCRIPT_LIST_MS
    );

    println!("bench_script_list: {}ms", duration.as_millis());
}

// ============================================
// Config Validation Performance Tests
// ============================================

#[test]
fn bench_config_validate() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
min_version = "0.6.0"

[project]
name = "validate-bench"

[tools]
node = "20"
uv = "latest"

[scripts]
dev = "npm run dev"

[scripts.build]
command = "npm run build"
depends = ["test"]

[hooks]
post_setup = "echo done"

[services.db]
image = "postgres:16"
ports = ["5432:5432"]
"#,
    );

    // Warm up
    let _ = env.run(&["config", "validate"]);

    // Measure
    let (output, duration) = env.run_timed(&["config", "validate"]);

    assert!(output.status.success() || !String::from_utf8_lossy(&output.stderr).is_empty());
    assert!(
        duration.as_millis() < thresholds::CONFIG_VALIDATE_MS as u128,
        "Config validate took {}ms, expected < {}ms",
        duration.as_millis(),
        thresholds::CONFIG_VALIDATE_MS
    );

    println!("bench_config_validate: {}ms", duration.as_millis());
}

// ============================================
// Repeated Operations Performance Tests
// ============================================

#[test]
fn bench_repeated_config_parse() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "repeated-bench"

[tools]
node = "20"

[scripts]
dev = "npm run dev"
"#,
    );

    // Warm up
    let _ = env.run(&["config", "show"]);

    // Measure 10 repeated operations
    let iterations = 10;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = env.run(&["config", "show"]);
    }

    let total_duration = start.elapsed();
    let avg_duration = total_duration / iterations;

    println!(
        "bench_repeated_config_parse: {}ms total, {}ms avg over {} iterations",
        total_duration.as_millis(),
        avg_duration.as_millis(),
        iterations
    );

    // Average should be reasonable
    assert!(
        avg_duration.as_millis() < thresholds::CONFIG_PARSE_SMALL_MS as u128,
        "Average config parse took {}ms, expected < {}ms",
        avg_duration.as_millis(),
        thresholds::CONFIG_PARSE_SMALL_MS
    );
}

#[test]
fn bench_repeated_setup_dryrun() {
    let env = E2ETestEnv::new();

    env.create_config(
        r#"
[project]
name = "repeated-setup"

[tools]
node = "20"
"#,
    );

    // Warm up
    let _ = env.run(&["setup", "--dry-run"]);

    // Measure 5 repeated operations
    let iterations = 5;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = env.run(&["setup", "--dry-run"]);
    }

    let total_duration = start.elapsed();
    let avg_duration = total_duration / iterations;

    println!(
        "bench_repeated_setup_dryrun: {}ms total, {}ms avg over {} iterations",
        total_duration.as_millis(),
        avg_duration.as_millis(),
        iterations
    );

    // Average should be reasonable
    assert!(
        avg_duration.as_millis() < thresholds::SETUP_DRYRUN_SMALL_MS as u128,
        "Average setup dry-run took {}ms, expected < {}ms",
        avg_duration.as_millis(),
        thresholds::SETUP_DRYRUN_SMALL_MS
    );
}

// ============================================
// Memory and Resource Tests
// ============================================

#[test]
fn bench_many_tools_config() {
    let env = E2ETestEnv::new();

    // Config with many tools
    let mut config = String::from("[project]\nname = \"many-tools\"\n\n[tools]\n");
    for i in 0..50 {
        config.push_str(&format!("tool{} = \"1.0.{}\"\n", i, i));
    }

    env.create_config(&config);

    let (output, duration) = env.run_timed(&["setup", "--dry-run"]);

    // Should handle many tools efficiently
    assert!(output.status.success());
    assert!(
        duration.as_millis() < 3000,
        "Many tools config took {}ms, expected < 3000ms",
        duration.as_millis()
    );

    println!("bench_many_tools_config: {}ms", duration.as_millis());
}

#[test]
fn bench_many_scripts_config() {
    let env = E2ETestEnv::new();

    // Config with many scripts
    let mut config = String::from("[project]\nname = \"many-scripts\"\n\n[scripts]\n");
    for i in 0..100 {
        config.push_str(&format!("script{} = \"echo {}\"\n", i, i));
    }

    env.create_config(&config);

    let (output, duration) = env.run_timed(&["run", "--list"]);

    // Should handle many scripts efficiently
    assert!(output.status.success() || !String::from_utf8_lossy(&output.stderr).is_empty());
    assert!(
        duration.as_millis() < 2000,
        "Many scripts config took {}ms, expected < 2000ms",
        duration.as_millis()
    );

    println!("bench_many_scripts_config: {}ms", duration.as_millis());
}

#[test]
fn bench_many_services_config() {
    let env = E2ETestEnv::new();

    // Config with many services
    let mut config = String::from("[project]\nname = \"many-services\"\n\n");
    for i in 0..20 {
        config.push_str(&format!(
            r#"
[services.service{}]
image = "alpine:{}"
ports = ["{}:{}"]
"#,
            i,
            i,
            8000 + i,
            80
        ));
    }

    env.create_config(&config);

    let (output, duration) = env.run_timed(&["services", "list"]);

    // Should handle many services efficiently
    assert!(output.status.success() || !String::from_utf8_lossy(&output.stderr).is_empty());
    assert!(
        duration.as_millis() < 2000,
        "Many services config took {}ms, expected < 2000ms",
        duration.as_millis()
    );

    println!("bench_many_services_config: {}ms", duration.as_millis());
}

// ============================================
// Summary Report
// ============================================

#[test]
fn bench_summary_report() {
    // This test just prints a summary header
    println!("\n========================================");
    println!("VX Performance Benchmark Results");
    println!("========================================");
    println!("Thresholds:");
    println!("  CLI Startup: < {}ms", thresholds::CLI_STARTUP_MS);
    println!("  Help Command: < {}ms", thresholds::HELP_MS);
    println!("  Version Command: < {}ms", thresholds::VERSION_MS);
    println!(
        "  Config Parse (small): < {}ms",
        thresholds::CONFIG_PARSE_SMALL_MS
    );
    println!(
        "  Config Parse (large): < {}ms",
        thresholds::CONFIG_PARSE_LARGE_MS
    );
    println!(
        "  Setup Dry-run (small): < {}ms",
        thresholds::SETUP_DRYRUN_SMALL_MS
    );
    println!(
        "  Setup Dry-run (large): < {}ms",
        thresholds::SETUP_DRYRUN_LARGE_MS
    );
    println!("  Script List: < {}ms", thresholds::SCRIPT_LIST_MS);
    println!("  Config Validate: < {}ms", thresholds::CONFIG_VALIDATE_MS);
    println!("========================================\n");
}
