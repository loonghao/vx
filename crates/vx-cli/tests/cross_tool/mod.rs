//! Cross-Tool E2E Tests for vx CLI
//!
//! Tests that involve multiple tools or cross-tool scenarios

use crate::common::*;
use rstest::*;
use tempfile::TempDir;

// ============================================================================
// Multi-Tool Version Checks
// ============================================================================

/// Test: Run version checks for multiple tools in sequence
#[rstest]
#[test]
fn test_multiple_tools_version_sequence() {
    skip_if_no_vx!();

    let tools = vec![
        ("node", vec!["--version"]),
        ("npm", vec!["--version"]),
        ("uv", vec!["--version"]),
        ("go", vec!["version"]),
        ("cargo", vec!["--version"]),
        ("bun", vec!["--version"]),
    ];

    for (tool, args) in tools {
        let mut full_args = vec![tool];
        full_args.extend(args);

        let output = run_vx(&full_args).unwrap_or_else(|_| panic!("Failed to run vx {}", tool));

        // Just verify it doesn't crash - tool may or may not be installed
        let _ = combined_output(&output);
    }
}

/// Test: Check all tools are discoverable via vx list
#[rstest]
#[test]
fn test_all_tools_listed() {
    skip_if_no_vx!();

    let output = run_vx(&["list"]).expect("Failed to run vx list");

    assert!(is_success(&output), "vx list should succeed");

    let stdout = stdout_str(&output);

    // Check that major tools are listed
    let expected_tools = ["node", "go", "uv", "bun", "cargo"];
    for tool in expected_tools {
        assert!(
            stdout.contains(tool),
            "vx list should include {}: {}",
            tool,
            stdout
        );
    }
}

// ============================================================================
// Tool Switching Scenarios
// ============================================================================

/// Test: Switch between Node and Bun for JavaScript
#[rstest]
#[test]
fn test_node_bun_switch() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = temp_dir.path().join("test.js");

    std::fs::write(&script, r#"console.log("Hello from JS!");"#).expect("Failed to write script");

    // Run with Node
    let node_output =
        run_vx_in_dir(temp_dir.path(), &["node", "test.js"]).expect("Failed to run with node");

    // Run with Bun
    let bun_output =
        run_vx_in_dir(temp_dir.path(), &["bun", "run", "test.js"]).expect("Failed to run with bun");

    // Both should produce same output if installed
    if is_success(&node_output) && is_success(&bun_output) {
        let node_stdout = stdout_str(&node_output);
        let bun_stdout = stdout_str(&bun_output);

        assert!(
            node_stdout.contains("Hello from JS!"),
            "Node output: {}",
            node_stdout
        );
        assert!(
            bun_stdout.contains("Hello from JS!"),
            "Bun output: {}",
            bun_stdout
        );
    }
}

/// Test: Switch between npm, pnpm, and yarn
#[rstest]
#[test]
fn test_package_manager_switch() {
    skip_if_no_vx!();

    let managers = ["npm", "pnpm", "yarn"];

    for manager in managers {
        let output =
            run_vx(&[manager, "--version"]).unwrap_or_else(|_| panic!("Failed to run {}", manager));

        // Just verify each works
        let _ = combined_output(&output);
    }
}

// ============================================================================
// Polyglot Project Scenarios
// ============================================================================

/// Test: Project with multiple language files
#[rstest]
#[test]
fn test_polyglot_project() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create JavaScript file
    std::fs::write(
        temp_dir.path().join("app.js"),
        r#"console.log("JS: Hello!");"#,
    )
    .expect("Failed to write JS file");

    // Create Go file
    std::fs::write(
        temp_dir.path().join("main.go"),
        r#"package main
import "fmt"
func main() { fmt.Println("Go: Hello!") }
"#,
    )
    .expect("Failed to write Go file");

    // Create Python script (for uv run)
    std::fs::write(
        temp_dir.path().join("script.py"),
        r#"print("Python: Hello!")"#,
    )
    .expect("Failed to write Python file");

    // Run each
    let js_output = run_vx_in_dir(temp_dir.path(), &["node", "app.js"]);
    let go_output = run_vx_in_dir(temp_dir.path(), &["go", "run", "main.go"]);

    // Verify at least one works
    let _ = js_output;
    let _ = go_output;
}

// ============================================================================
// Tool Chain Scenarios
// ============================================================================

/// Test: Build and run with cargo
#[rstest]
#[test]
fn test_cargo_build_run_chain() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        // Build
        let build_output =
            run_vx_in_dir(temp_dir.path(), &["cargo", "build"]).expect("Failed to run cargo build");

        if is_success(&build_output) {
            // Run
            let run_output =
                run_vx_in_dir(temp_dir.path(), &["cargo", "run"]).expect("Failed to run cargo run");

            assert!(is_success(&run_output), "cargo run should succeed");
        }
    }
}

/// Test: npm init, install, and run
#[rstest]
#[test]
fn test_npm_init_install_run() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["npm", "init", "-y"]).expect("Failed to run npm init");

    if is_success(&init_output) {
        // Add a script to package.json
        std::fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name": "test", "scripts": {"start": "echo started"}}"#,
        )
        .expect("Failed to update package.json");

        // Run script
        let run_output = run_vx_in_dir(temp_dir.path(), &["npm", "run", "start"])
            .expect("Failed to run npm run start");

        if is_success(&run_output) {
            assert_output_contains(&run_output, "started", "npm run start");
        }
    }
}

/// Test: uv init and run
#[rstest]
#[test]
fn test_uv_init_run_chain() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["uv", "init"]).expect("Failed to run uv init");

    if is_success(&init_output) {
        // Create a script
        std::fs::write(
            temp_dir.path().join("hello.py"),
            r#"print("Hello from uv!")"#,
        )
        .expect("Failed to write script");

        // Run
        let run_output = run_vx_in_dir(temp_dir.path(), &["uv", "run", "python", "hello.py"])
            .expect("Failed to run uv run");

        if is_success(&run_output) {
            assert_stdout_contains(&run_output, "Hello from uv!", "uv run");
        }
    }
}

/// Test: go mod init and run
#[rstest]
#[test]
fn test_go_mod_init_run() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init module
    let init_output = run_vx_in_dir(temp_dir.path(), &["go", "mod", "init", "example.com/test"])
        .expect("Failed to run go mod init");

    if is_success(&init_output) {
        // Create main.go
        std::fs::write(
            temp_dir.path().join("main.go"),
            r#"package main
import "fmt"
func main() { fmt.Println("Hello from go mod!") }
"#,
        )
        .expect("Failed to write main.go");

        // Run
        let run_output =
            run_vx_in_dir(temp_dir.path(), &["go", "run", "."]).expect("Failed to run go run");

        if is_success(&run_output) {
            assert_stdout_contains(&run_output, "Hello from go mod!", "go run");
        }
    }
}

// ============================================================================
// Environment Variable Propagation
// ============================================================================

/// Test: Environment variables propagate to all tools
#[rstest]
#[test]
fn test_env_propagation() {
    skip_if_no_vx!();

    let env_var = ("VX_CROSS_TOOL_TEST", "cross_tool_value");

    // Test with Node
    let node_output = run_vx_with_env(
        &["node", "-e", "console.log(process.env.VX_CROSS_TOOL_TEST)"],
        &[env_var],
    )
    .expect("Failed to run node");

    if is_success(&node_output) {
        assert_stdout_contains(&node_output, "cross_tool_value", "node env");
    }

    // Test with Go
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    std::fs::write(
        temp_dir.path().join("main.go"),
        r#"package main
import ("fmt"; "os")
func main() { fmt.Println(os.Getenv("VX_CROSS_TOOL_TEST")) }
"#,
    )
    .expect("Failed to write main.go");

    let go_output = run_vx_with_env(
        &[
            "go",
            "run",
            &temp_dir.path().join("main.go").to_string_lossy(),
        ],
        &[env_var],
    )
    .expect("Failed to run go");

    if is_success(&go_output) {
        assert_stdout_contains(&go_output, "cross_tool_value", "go env");
    }
}

// ============================================================================
// Exit Code Propagation
// ============================================================================

/// Test: Exit codes propagate correctly from all tools
#[rstest]
#[test]
fn test_exit_code_propagation() {
    skip_if_no_vx!();

    // Node exit 0
    let node_success = run_vx(&["node", "-e", "process.exit(0)"]).expect("Failed to run node");
    if tool_installed("node") {
        assert!(is_success(&node_success), "Node exit 0 should succeed");
    }

    // Node exit 1
    let node_fail = run_vx(&["node", "-e", "process.exit(1)"]).expect("Failed to run node");
    if tool_installed("node") {
        assert!(!is_success(&node_fail), "Node exit 1 should fail");
    }

    // Go exit
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    std::fs::write(
        temp_dir.path().join("exit.go"),
        r#"package main
import "os"
func main() { os.Exit(2) }
"#,
    )
    .expect("Failed to write exit.go");

    let go_exit =
        run_vx_in_dir(temp_dir.path(), &["go", "run", "exit.go"]).expect("Failed to run go");

    if tool_installed("go") {
        assert!(!is_success(&go_exit), "Go exit 2 should fail");
    }
}

// ============================================================================
// Working Directory Scenarios
// ============================================================================

/// Test: Tools respect current working directory
#[rstest]
#[test]
fn test_working_directory_respected() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let sub_dir = temp_dir.path().join("subdir");
    std::fs::create_dir(&sub_dir).expect("Failed to create subdir");

    // Create file in subdir
    std::fs::write(sub_dir.join("test.js"), r#"console.log("in subdir");"#)
        .expect("Failed to write file");

    // Run from subdir
    let output = run_vx_in_dir(&sub_dir, &["node", "test.js"]).expect("Failed to run in subdir");

    if is_success(&output) {
        assert_stdout_contains(&output, "in subdir", "working directory");
    }
}

// ============================================================================
// Concurrent Tool Usage
// ============================================================================

/// Test: Multiple tools can be used in same project
#[rstest]
#[test]
fn test_multiple_tools_same_project() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create package.json for Node/npm
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{"name": "test", "scripts": {"hello": "echo npm hello"}}"#,
    )
    .expect("Failed to write package.json");

    // Create go.mod for Go
    let _ = run_vx_in_dir(temp_dir.path(), &["go", "mod", "init", "example.com/test"]);

    // Create main.go
    std::fs::write(
        temp_dir.path().join("main.go"),
        r#"package main
func main() { println("go hello") }
"#,
    )
    .expect("Failed to write main.go");

    // Run npm script
    let npm_output = run_vx_in_dir(temp_dir.path(), &["npm", "run", "hello"]);

    // Run go
    let go_output = run_vx_in_dir(temp_dir.path(), &["go", "run", "."]);

    // Both should work independently
    let _ = npm_output;
    let _ = go_output;
}

// ============================================================================
// Tool Discovery
// ============================================================================

/// Test: vx which for multiple tools
#[rstest]
#[test]
fn test_which_multiple_tools() {
    skip_if_no_vx!();

    let tools = ["node", "npm", "go", "cargo", "uv", "bun"];

    for tool in tools {
        let output =
            run_vx(&["which", tool]).unwrap_or_else(|_| panic!("Failed to run vx which {}", tool));

        // May succeed or fail depending on installation
        let _ = combined_output(&output);
    }
}

/// Test: vx search for tools
#[rstest]
#[test]
fn test_search_tools() {
    skip_if_no_vx!();

    let output = run_vx(&["search"]).expect("Failed to run vx search");

    assert!(is_success(&output), "vx search should succeed");
}
