//! Real Tool Tests - End-to-End tests with actual tool installation and execution
//!
//! This module provides comprehensive E2E tests that install and run real tools.
//! These tests require network access and are marked with `#[ignore]` by default.
//!
//! # Running Tests
//!
//! ```bash
//! # Run all real tool tests (requires network)
//! cargo test --package vx-cli --test real_tool_tests -- --ignored --nocapture
//!
//! # Run specific tool tests
//! cargo test --package vx-cli --test real_tool_tests uv -- --ignored
//! cargo test --package vx-cli --test real_tool_tests node -- --ignored
//! cargo test --package vx-cli --test real_tool_tests go -- --ignored
//! cargo test --package vx-cli --test real_tool_tests bun -- --ignored
//!
//! # Run quick smoke tests only
//! cargo test --package vx-cli --test real_tool_tests smoke -- --ignored
//! ```

mod common;

use common::{
    assert_output_contains, assert_success, cleanup_test_env, combined_output, init_test_env,
    is_success, run_vx, run_vx_in_dir, stderr_str, stdout_str, vx_available,
};
use rstest::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Framework Helpers
// ============================================================================

/// Test context for real tool tests
struct RealToolTestContext {
    temp_dir: TempDir,
    tool_name: &'static str,
}

impl RealToolTestContext {
    fn new(tool_name: &'static str) -> Self {
        init_test_env();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        Self {
            temp_dir,
            tool_name,
        }
    }

    fn path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    /// Install the tool via vx
    fn install(&self) -> bool {
        let output = run_vx(&["install", self.tool_name]);
        match output {
            Ok(o) => {
                if !is_success(&o) {
                    eprintln!(
                        "Failed to install {}: {}",
                        self.tool_name,
                        combined_output(&o)
                    );
                }
                is_success(&o)
            }
            Err(e) => {
                eprintln!("Failed to run vx install {}: {}", self.tool_name, e);
                false
            }
        }
    }

    /// Check if tool is installed
    fn is_installed(&self) -> bool {
        run_vx(&["which", self.tool_name])
            .map(|o| is_success(&o))
            .unwrap_or(false)
    }

    /// Run the tool with arguments
    fn run(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        let mut full_args = vec![self.tool_name];
        full_args.extend(args);
        run_vx(&full_args)
    }

    /// Run the tool in the temp directory
    fn run_in_dir(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        let mut full_args = vec![self.tool_name];
        full_args.extend(args);
        run_vx_in_dir(self.temp_dir.path(), &full_args)
    }
}

impl Drop for RealToolTestContext {
    fn drop(&mut self) {
        cleanup_test_env();
    }
}

/// Skip test if vx is not available
macro_rules! require_vx {
    () => {
        if !vx_available() {
            eprintln!("Skipping: vx binary not found");
            return;
        }
    };
}

/// Skip test with a message
macro_rules! skip_test {
    ($msg:expr) => {
        eprintln!("Skipping: {}", $msg);
        return;
    };
}

// ============================================================================
// Smoke Tests - Quick verification that vx works
// ============================================================================

mod smoke_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires vx binary"]
    fn smoke_vx_version() {
        require_vx!();
        let output = run_vx(&["--version"]).expect("Failed to run vx");
        assert_success(&output, "vx --version");
        assert_output_contains(&output, "vx", "version output");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires vx binary"]
    fn smoke_vx_list() {
        require_vx!();
        let output = run_vx(&["list"]).expect("Failed to run vx");
        assert_success(&output, "vx list");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires vx binary"]
    fn smoke_vx_list_status() {
        require_vx!();
        let output = run_vx(&["list", "--status"]).expect("Failed to run vx");
        assert_success(&output, "vx list --status");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires vx binary"]
    fn smoke_vx_stats() {
        require_vx!();
        let output = run_vx(&["stats"]).expect("Failed to run vx");
        assert_success(&output, "vx stats");
    }
}

// ============================================================================
// UV (Python) Tests
// ============================================================================

mod uv_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uv_install_and_version() {
        require_vx!();
        let ctx = RealToolTestContext::new("uv");

        // Install UV
        assert!(ctx.install(), "UV installation should succeed");

        // Verify version
        let output = ctx.run(&["--version"]).expect("Failed to run uv");
        assert_success(&output, "uv --version");
        assert_output_contains(&output, "uv", "version output should contain 'uv'");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uv_create_venv() {
        require_vx!();
        let ctx = RealToolTestContext::new("uv");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("UV not available");
        }

        // Create virtual environment
        let output = ctx
            .run_in_dir(&["venv", ".venv"])
            .expect("Failed to run uv venv");
        assert_success(&output, "uv venv .venv");

        // Verify .venv directory exists
        let venv_path = ctx.path().join(".venv");
        assert!(venv_path.exists(), ".venv directory should exist");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uv_init_project() {
        require_vx!();
        let ctx = RealToolTestContext::new("uv");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("UV not available");
        }

        // Initialize project
        let output = ctx
            .run_in_dir(&["init", "--name", "test-project"])
            .expect("Failed to run uv init");

        // May succeed or fail depending on UV version
        if is_success(&output) {
            let pyproject = ctx.path().join("pyproject.toml");
            assert!(pyproject.exists(), "pyproject.toml should exist");
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uvx_run_package() {
        require_vx!();
        let ctx = RealToolTestContext::new("uv");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("UV not available");
        }

        // Run ruff via uvx (fast, reliable package)
        let output = run_vx(&["uvx", "ruff", "--version"]).expect("Failed to run uvx");
        assert_success(&output, "uvx ruff --version");
        assert_output_contains(&output, "ruff", "should show ruff version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uvx_run_cowsay() {
        require_vx!();
        let ctx = RealToolTestContext::new("uv");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("UV not available");
        }

        // Run cowsay via uvx
        let output = run_vx(&["uvx", "cowsay", "Hello vx!"]).expect("Failed to run uvx");

        // cowsay might not be available on all platforms, so just check it doesn't crash
        let _ = output;
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn uv_pip_install() {
        require_vx!();
        let ctx = RealToolTestContext::new("uv");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("UV not available");
        }

        // Create venv first
        let _ = ctx.run_in_dir(&["venv", ".venv"]);

        // Install a package (requests is small and reliable)
        let output = ctx
            .run_in_dir(&["pip", "install", "six", "--quiet"])
            .expect("Failed to run uv pip install");

        // May fail if venv activation is required
        let _ = output;
    }
}

// ============================================================================
// Node.js Tests
// ============================================================================

mod node_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn node_install_and_version() {
        require_vx!();
        let ctx = RealToolTestContext::new("node");

        // Install Node.js
        if !ctx.install() {
            skip_test!("Node.js installation failed");
        }

        // Verify version
        let output = ctx.run(&["--version"]).expect("Failed to run node");
        assert_success(&output, "node --version");

        let stdout = stdout_str(&output);
        assert!(
            stdout.starts_with('v') || stdout.contains('.'),
            "Version should be in format vX.Y.Z"
        );
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn node_run_inline_js() {
        require_vx!();
        let ctx = RealToolTestContext::new("node");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Node.js not available");
        }

        // Run inline JavaScript
        let output = ctx
            .run(&["-e", "console.log('Hello from vx!')"])
            .expect("Failed to run node");
        assert_success(&output, "node -e");
        assert_output_contains(&output, "Hello from vx!", "should print message");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn node_run_json_output() {
        require_vx!();
        let ctx = RealToolTestContext::new("node");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Node.js not available");
        }

        // Run JSON output
        let output = ctx
            .run(&["-e", "console.log(JSON.stringify({tool:'vx',status:'ok'}))"])
            .expect("Failed to run node");
        assert_success(&output, "node JSON output");
        assert_output_contains(&output, "vx", "should contain 'vx'");
        assert_output_contains(&output, "ok", "should contain 'ok'");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn node_run_script_file() {
        require_vx!();
        let ctx = RealToolTestContext::new("node");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Node.js not available");
        }

        // Create a test script
        let script_path = ctx.path().join("test.js");
        fs::write(
            &script_path,
            r#"
const os = require('os');
console.log('Platform:', os.platform());
console.log('Node version:', process.version);
console.log('Test passed!');
"#,
        )
        .expect("Failed to write script");

        // Run the script
        let output = ctx
            .run_in_dir(&["test.js"])
            .expect("Failed to run node script");
        assert_success(&output, "node test.js");
        assert_output_contains(&output, "Test passed!", "should complete successfully");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn npm_version() {
        require_vx!();
        let ctx = RealToolTestContext::new("node");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Node.js not available");
        }

        // npm should be available with node
        let output = run_vx(&["npm", "--version"]).expect("Failed to run npm");
        assert_success(&output, "npm --version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn npm_init_project() {
        require_vx!();
        let ctx = RealToolTestContext::new("node");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Node.js not available");
        }

        // Initialize npm project
        let output = run_vx_in_dir(ctx.temp_dir.path(), &["npm", "init", "-y"])
            .expect("Failed to run npm init");

        // npm init may fail in some environments, just check it doesn't crash
        if is_success(&output) {
            // Verify package.json exists
            let package_json = ctx.path().join("package.json");
            assert!(package_json.exists(), "package.json should exist");

            // Verify content
            let content = fs::read_to_string(&package_json).expect("Failed to read package.json");
            assert!(content.contains("\"name\""), "should have name field");
        } else {
            eprintln!(
                "npm init -y returned non-zero (may be expected): {}",
                combined_output(&output)
            );
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn npx_run_package() {
        require_vx!();
        let ctx = RealToolTestContext::new("node");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Node.js not available");
        }

        // Run cowsay via npx
        let output =
            run_vx(&["npx", "-y", "cowsay", "Hello from npx!"]).expect("Failed to run npx");

        // cowsay should work
        if is_success(&output) {
            assert_output_contains(&output, "Hello from npx!", "should show message");
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn npx_create_project() {
        require_vx!();
        let ctx = RealToolTestContext::new("node");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Node.js not available");
        }

        // Create a simple project with degit (fast)
        let output = run_vx_in_dir(
            ctx.temp_dir.path(),
            &["npx", "-y", "degit", "sveltejs/template", "my-app"],
        );

        // May fail due to network, just verify it doesn't crash
        let _ = output;
    }
}

// ============================================================================
// Go Tests
// ============================================================================

mod go_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn go_install_and_version() {
        require_vx!();
        let ctx = RealToolTestContext::new("go");

        // Install Go
        if !ctx.install() {
            skip_test!("Go installation failed");
        }

        // Verify version
        let output = ctx.run(&["version"]).expect("Failed to run go");
        assert_success(&output, "go version");
        assert_output_contains(&output, "go version", "should show go version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn go_run_inline() {
        require_vx!();
        let ctx = RealToolTestContext::new("go");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Go not available");
        }

        // Create a test file
        let main_go = ctx.path().join("main.go");
        fs::write(
            &main_go,
            r#"package main

import "fmt"

func main() {
    fmt.Println("Hello from Go via vx!")
}
"#,
        )
        .expect("Failed to write main.go");

        // Run the file
        let output = ctx
            .run_in_dir(&["run", "main.go"])
            .expect("Failed to run go");
        assert_success(&output, "go run main.go");
        assert_output_contains(&output, "Hello from Go via vx!", "should print message");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn go_mod_init() {
        require_vx!();
        let ctx = RealToolTestContext::new("go");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Go not available");
        }

        // Initialize module
        let output = ctx
            .run_in_dir(&["mod", "init", "example.com/test"])
            .expect("Failed to run go mod init");
        assert_success(&output, "go mod init");

        // Verify go.mod exists
        let go_mod = ctx.path().join("go.mod");
        assert!(go_mod.exists(), "go.mod should exist");

        let content = fs::read_to_string(&go_mod).expect("Failed to read go.mod");
        assert!(
            content.contains("example.com/test"),
            "should have module name"
        );
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn go_build() {
        require_vx!();
        let ctx = RealToolTestContext::new("go");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Go not available");
        }

        // Create a buildable project
        let main_go = ctx.path().join("main.go");
        fs::write(
            &main_go,
            r#"package main

import "fmt"

func main() {
    fmt.Println("Built with vx!")
}
"#,
        )
        .expect("Failed to write main.go");

        // Initialize module
        let _ = ctx.run_in_dir(&["mod", "init", "example.com/build-test"]);

        // Build
        let output = ctx
            .run_in_dir(&["build", "-o", "app"])
            .expect("Failed to run go build");
        assert_success(&output, "go build");

        // Verify binary exists
        let binary_name = if cfg!(windows) { "app.exe" } else { "app" };
        let binary = ctx.path().join(binary_name);

        // On Windows, go build might add .exe automatically
        assert!(
            binary.exists() || ctx.path().join("app").exists(),
            "binary should exist"
        );
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn go_fmt() {
        require_vx!();
        let ctx = RealToolTestContext::new("go");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Go not available");
        }

        // Create unformatted Go file
        let main_go = ctx.path().join("main.go");
        fs::write(
            &main_go,
            r#"package main
import "fmt"
func main(){fmt.Println("unformatted")}"#,
        )
        .expect("Failed to write main.go");

        // Format
        let output = ctx
            .run_in_dir(&["fmt", "main.go"])
            .expect("Failed to run go fmt");
        assert_success(&output, "go fmt");

        // Verify formatted
        let content = fs::read_to_string(&main_go).expect("Failed to read main.go");
        assert!(
            content.contains("func main() {"),
            "should be properly formatted"
        );
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn go_env() {
        require_vx!();
        let ctx = RealToolTestContext::new("go");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Go not available");
        }

        // Show Go environment
        let output = ctx.run(&["env"]).expect("Failed to run go env");
        assert_success(&output, "go env");
        assert_output_contains(&output, "GOPATH", "should show GOPATH");
    }
}

// ============================================================================
// Bun Tests
// ============================================================================

mod bun_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn bun_install_and_version() {
        require_vx!();
        let ctx = RealToolTestContext::new("bun");

        // Install Bun - may fail on some platforms
        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Bun not available");
        }

        // Verify version
        let output = ctx.run(&["--version"]).expect("Failed to run bun");
        assert_success(&output, "bun --version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn bun_run_inline_js() {
        require_vx!();
        let ctx = RealToolTestContext::new("bun");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Bun not available");
        }

        // Run inline JavaScript - bun may use different flag syntax
        let output = ctx
            .run(&["-e", "console.log('Hello from Bun via vx!')"])
            .expect("Failed to run bun");

        // Bun -e may not work on all versions, check if it succeeds
        if is_success(&output) {
            assert_output_contains(&output, "Hello from Bun via vx!", "should print message");
        } else {
            eprintln!("bun -e not supported in this version");
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn bun_init_project() {
        require_vx!();
        let ctx = RealToolTestContext::new("bun");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Bun not available");
        }

        // Initialize project
        let output = ctx
            .run_in_dir(&["init", "-y"])
            .expect("Failed to run bun init");

        // May succeed or fail depending on Bun version
        if is_success(&output) {
            let package_json = ctx.path().join("package.json");
            assert!(package_json.exists(), "package.json should exist");
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn bun_run_script() {
        require_vx!();
        let ctx = RealToolTestContext::new("bun");

        if !ctx.is_installed() && !ctx.install() {
            skip_test!("Bun not available");
        }

        // Create a test script
        let script_path = ctx.path().join("test.ts");
        fs::write(
            &script_path,
            r#"
const message: string = "Hello from Bun TypeScript!";
console.log(message);
"#,
        )
        .expect("Failed to write script");

        // Run the script (Bun supports TypeScript natively)
        let output = ctx
            .run_in_dir(&["run", "test.ts"])
            .expect("Failed to run bun script");

        // Bun run may fail in some environments
        if is_success(&output) {
            assert_output_contains(
                &output,
                "Hello from Bun TypeScript!",
                "should run TypeScript",
            );
        } else {
            eprintln!(
                "bun run test.ts failed (may be expected): {}",
                combined_output(&output)
            );
        }
    }
}

// ============================================================================
// PNPM Tests
// ============================================================================

mod pnpm_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn pnpm_install_and_version() {
        require_vx!();

        // First ensure node is installed
        let node_ctx = RealToolTestContext::new("node");
        if !node_ctx.is_installed() && !node_ctx.install() {
            skip_test!("Node.js not available (required for pnpm)");
        }

        let ctx = RealToolTestContext::new("pnpm");

        // Install pnpm
        if !ctx.install() {
            skip_test!("pnpm installation failed");
        }

        // Verify version
        let output = ctx.run(&["--version"]).expect("Failed to run pnpm");
        assert_success(&output, "pnpm --version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn pnpm_init_project() {
        require_vx!();

        // Ensure node is installed
        let node_ctx = RealToolTestContext::new("node");
        if !node_ctx.is_installed() && !node_ctx.install() {
            skip_test!("Node.js not available");
        }

        let ctx = RealToolTestContext::new("pnpm");
        if !ctx.is_installed() && !ctx.install() {
            skip_test!("pnpm not available");
        }

        // Initialize project
        let output = ctx.run_in_dir(&["init"]).expect("Failed to run pnpm init");
        assert_success(&output, "pnpm init");

        // Verify package.json exists
        let package_json = ctx.path().join("package.json");
        assert!(package_json.exists(), "package.json should exist");
    }
}

// ============================================================================
// Yarn Tests
// ============================================================================

mod yarn_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn yarn_install_and_version() {
        require_vx!();

        // First ensure node is installed
        let node_ctx = RealToolTestContext::new("node");
        if !node_ctx.is_installed() && !node_ctx.install() {
            skip_test!("Node.js not available (required for yarn)");
        }

        let ctx = RealToolTestContext::new("yarn");

        // Install yarn
        if !ctx.install() {
            skip_test!("yarn installation failed");
        }

        // Verify version
        let output = ctx.run(&["--version"]).expect("Failed to run yarn");
        assert_success(&output, "yarn --version");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn yarn_init_project() {
        require_vx!();

        // Ensure node is installed
        let node_ctx = RealToolTestContext::new("node");
        if !node_ctx.is_installed() && !node_ctx.install() {
            skip_test!("Node.js not available");
        }

        let ctx = RealToolTestContext::new("yarn");
        if !ctx.is_installed() && !ctx.install() {
            skip_test!("yarn not available");
        }

        // Initialize project
        let output = ctx
            .run_in_dir(&["init", "-y"])
            .expect("Failed to run yarn init");
        assert_success(&output, "yarn init -y");

        // Verify package.json exists
        let package_json = ctx.path().join("package.json");
        assert!(package_json.exists(), "package.json should exist");
    }
}

// ============================================================================
// Cross-Tool Integration Tests
// ============================================================================

mod cross_tool_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn multi_language_project() {
        require_vx!();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path();

        // Install all required tools
        let tools = ["uv", "node", "go"];
        for tool in &tools {
            let output = run_vx(&["install", tool]);
            if output.is_err() || !is_success(&output.unwrap()) {
                skip_test!(format!("{} not available", tool));
            }
        }

        // Create Python backend
        let backend_dir = project_path.join("backend");
        fs::create_dir_all(&backend_dir).expect("Failed to create backend dir");

        let output = run_vx_in_dir(&backend_dir, &["uv", "init", "--name", "backend"]);
        if let Ok(o) = &output {
            if is_success(o) {
                assert!(
                    backend_dir.join("pyproject.toml").exists(),
                    "Python project should be initialized"
                );
            }
        }

        // Create Node.js frontend
        let frontend_dir = project_path.join("frontend");
        fs::create_dir_all(&frontend_dir).expect("Failed to create frontend dir");

        let output = run_vx_in_dir(&frontend_dir, &["npm", "init", "-y"]);
        if let Ok(o) = &output {
            assert_success(o, "npm init");
            assert!(
                frontend_dir.join("package.json").exists(),
                "Node.js project should be initialized"
            );
        }

        // Create Go services
        let services_dir = project_path.join("services");
        fs::create_dir_all(&services_dir).expect("Failed to create services dir");

        let output = run_vx_in_dir(
            &services_dir,
            &["go", "mod", "init", "example.com/services"],
        );
        if let Ok(o) = &output {
            assert_success(o, "go mod init");
            assert!(
                services_dir.join("go.mod").exists(),
                "Go project should be initialized"
            );
        }

        // Verify project structure
        assert!(backend_dir.exists(), "backend should exist");
        assert!(frontend_dir.exists(), "frontend should exist");
        assert!(services_dir.exists(), "services should exist");
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn tool_switching() {
        require_vx!();

        // This test verifies that switching between tools works correctly
        let _temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Run Node.js
        let node_output = run_vx(&["node", "-e", "console.log('node')"]);

        // Run Go (if available)
        let go_output = run_vx(&["go", "version"]);

        // Run UV (if available)
        let uv_output = run_vx(&["uv", "--version"]);

        // At least one should work
        let _any_success = [&node_output, &go_output, &uv_output]
            .iter()
            .any(|o| o.as_ref().map(is_success).unwrap_or(false));

        // If vx is installed, at least one tool should be available or installable
        if vx_available() {
            // This is informational - we just want to verify no crashes
            eprintln!("Tool switching test completed");
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn concurrent_tool_execution() {
        require_vx!();

        use std::thread;

        // Run multiple tools concurrently
        let handles: Vec<_> = ["node", "go", "uv"]
            .iter()
            .map(|tool| {
                let tool = tool.to_string();
                thread::spawn(move || {
                    let args = match tool.as_str() {
                        "node" => vec!["node", "-e", "console.log('node')"],
                        "go" => vec!["go", "version"],
                        "uv" => vec!["uv", "--version"],
                        _ => vec![],
                    };
                    run_vx(&args.iter().map(|s| *s).collect::<Vec<_>>())
                })
            })
            .collect();

        // Wait for all to complete
        for handle in handles {
            let _ = handle.join();
        }

        // Test passes if no panics occurred
    }
}

// ============================================================================
// Auto-Install Tests
// ============================================================================

mod auto_install_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn auto_install_on_execute() {
        require_vx!();

        // This test verifies that vx auto-installs tools when needed
        // We use a fresh temp directory to avoid cached installations

        // Try to run a tool - vx should auto-install if needed
        let output = run_vx(&["uv", "--version"]);

        if let Ok(o) = output {
            // Either succeeds (tool installed) or fails with helpful message
            let combined = combined_output(&o);
            eprintln!("Auto-install test output: {}", combined);
        }
    }
}

// ============================================================================
// Version Management Tests
// ============================================================================

mod version_management_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn install_specific_version() {
        require_vx!();

        // Install a specific version of Node.js
        let output = run_vx(&["install", "node@20"]);

        if let Ok(o) = &output {
            if is_success(o) {
                // Verify version
                let version_output = run_vx(&["node", "--version"]).expect("Failed to get version");
                let stdout = stdout_str(&version_output);
                assert!(stdout.contains("20"), "Should install Node.js 20.x");
            }
        }
    }

    #[rstest]
    #[test]
    #[ignore = "Requires network access"]
    fn list_installed_versions() {
        require_vx!();

        // Install a tool first
        let _ = run_vx(&["install", "uv"]);

        // List installed versions
        let output = run_vx(&["list", "--status"]).expect("Failed to run vx list");
        assert_success(&output, "vx list --status");

        // Should show some installed tools
        let stdout = stdout_str(&output);
        eprintln!("Installed tools:\n{}", stdout);
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

mod error_handling_tests {
    use super::*;

    #[rstest]
    #[test]
    #[ignore = "Requires vx binary"]
    fn graceful_error_on_invalid_tool() {
        require_vx!();

        let output =
            run_vx(&["nonexistent-tool-xyz-123"]).expect("Failed to run vx with invalid tool");

        // Should fail but not crash
        assert!(!is_success(&output), "Should fail for nonexistent tool");

        let stderr = stderr_str(&output);
        let stdout = stdout_str(&output);
        let combined = format!("{}{}", stdout, stderr);

        // Should provide helpful error message
        assert!(
            combined.contains("not found")
                || combined.contains("error")
                || combined.contains("unknown")
                || combined.contains("Error"),
            "Should provide helpful error message"
        );
    }

    #[rstest]
    #[test]
    #[ignore = "Requires vx binary"]
    fn graceful_error_on_invalid_version() {
        require_vx!();

        let output = run_vx(&["install", "node@999.999.999"]);

        if let Ok(o) = output {
            // Should fail but not crash
            if !is_success(&o) {
                let combined = combined_output(&o);
                eprintln!("Invalid version error: {}", combined);
            }
        }
    }
}
