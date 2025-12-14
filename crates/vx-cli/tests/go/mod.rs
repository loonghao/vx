//! Go E2E Tests for vx CLI
//!
//! Tests for Go toolchain

use crate::common::*;
use rstest::*;
use tempfile::TempDir;

// ============================================================================
// Go Version Tests
// ============================================================================

/// Test: vx go version
#[rstest]
#[test]
fn test_go_version() {
    skip_if_no_vx!();

    let output = run_vx(&["go", "version"]).expect("Failed to run vx go version");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.contains("go version"),
            "go version should contain 'go version': {}",
            version
        );
    }
}

// ============================================================================
// Go Env Tests
// ============================================================================

/// Test: vx go env
#[rstest]
#[test]
fn test_go_env() {
    skip_if_no_vx!();

    let output = run_vx(&["go", "env"]).expect("Failed to run vx go env");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("GOPATH") || stdout.contains("GOROOT"),
            "go env should show environment: {}",
            stdout
        );
    }
}

/// Test: vx go env GOVERSION
#[rstest]
#[test]
fn test_go_env_goversion() {
    skip_if_no_vx!();

    let output = run_vx(&["go", "env", "GOVERSION"]).expect("Failed to run vx go env GOVERSION");

    if is_success(&output) {
        let version = stdout_str(&output).trim().to_string();
        assert!(
            version.starts_with("go"),
            "GOVERSION should start with 'go': {}",
            version
        );
    }
}

/// Test: vx go env GOROOT
#[rstest]
#[test]
fn test_go_env_goroot() {
    skip_if_no_vx!();

    let output = run_vx(&["go", "env", "GOROOT"]).expect("Failed to run vx go env GOROOT");

    if is_success(&output) {
        let goroot = stdout_str(&output).trim().to_string();
        assert!(!goroot.is_empty(), "GOROOT should not be empty");
    }
}

/// Test: vx go env GOPATH
#[rstest]
#[test]
fn test_go_env_gopath() {
    skip_if_no_vx!();

    let output = run_vx(&["go", "env", "GOPATH"]).expect("Failed to run vx go env GOPATH");

    if is_success(&output) {
        let gopath = stdout_str(&output).trim().to_string();
        assert!(!gopath.is_empty(), "GOPATH should not be empty");
    }
}

/// Test: vx go env -json
#[rstest]
#[test]
fn test_go_env_json() {
    skip_if_no_vx!();

    let output = run_vx(&["go", "env", "-json"]).expect("Failed to run vx go env -json");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("{") && stdout.contains("}"),
            "go env -json should output JSON: {}",
            stdout
        );
    }
}

// ============================================================================
// Go Help Tests
// ============================================================================

/// Test: vx go help
#[rstest]
#[test]
fn test_go_help() {
    skip_if_no_vx!();

    let output = run_vx(&["go", "help"]).expect("Failed to run vx go help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("Go is a tool") || stdout.contains("Usage"),
            "go help should show usage: {}",
            stdout
        );
    }
}

/// Test: vx go help build
#[rstest]
#[test]
fn test_go_help_build() {
    skip_if_no_vx!();

    let output = run_vx(&["go", "help", "build"]).expect("Failed to run vx go help build");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("build") || stdout.contains("compile"),
            "go help build: {}",
            stdout
        );
    }
}

// ============================================================================
// Go Run Tests
// ============================================================================

/// Test: vx go run with simple program
#[rstest]
#[test]
fn test_go_run_hello() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let main_go = temp_dir.path().join("main.go");

    std::fs::write(
        &main_go,
        r#"package main

import "fmt"

func main() {
    fmt.Println("Hello from vx go!")
}
"#,
    )
    .expect("Failed to write main.go");

    let output =
        run_vx_in_dir(temp_dir.path(), &["go", "run", "main.go"]).expect("Failed to run go run");

    if is_success(&output) {
        assert_stdout_contains(&output, "Hello from vx go!", "go run");
    }
}

/// Test: vx go run with arguments
#[rstest]
#[test]
fn test_go_run_with_args() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let main_go = temp_dir.path().join("main.go");

    std::fs::write(
        &main_go,
        r#"package main

import (
    "fmt"
    "os"
    "strings"
)

func main() {
    fmt.Println("Args:", strings.Join(os.Args[1:], ", "))
}
"#,
    )
    .expect("Failed to write main.go");

    let output = run_vx_in_dir(temp_dir.path(), &["go", "run", "main.go", "arg1", "arg2"])
        .expect("Failed to run go run with args");

    if is_success(&output) {
        assert_stdout_contains(&output, "arg1, arg2", "go run args");
    }
}

/// Test: vx go run with environment variable
#[rstest]
#[test]
fn test_go_run_with_env() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let main_go = temp_dir.path().join("main.go");

    std::fs::write(
        &main_go,
        r#"package main

import (
    "fmt"
    "os"
)

func main() {
    fmt.Println(os.Getenv("VX_TEST_VAR"))
}
"#,
    )
    .expect("Failed to write main.go");

    let output = run_vx_with_env(
        &["go", "run", &main_go.to_string_lossy()],
        &[("VX_TEST_VAR", "test_value_go")],
    )
    .expect("Failed to run go run with env");

    if is_success(&output) {
        assert_stdout_contains(&output, "test_value_go", "go run env");
    }
}

// ============================================================================
// Go Build Tests
// ============================================================================

/// Test: vx go build
#[rstest]
#[test]
fn test_go_build() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let main_go = temp_dir.path().join("main.go");

    std::fs::write(
        &main_go,
        r#"package main

func main() {
    println("built!")
}
"#,
    )
    .expect("Failed to write main.go");

    let output = run_vx_in_dir(temp_dir.path(), &["go", "build", "-o", "app", "main.go"])
        .expect("Failed to run go build");

    if is_success(&output) {
        let binary_name = if cfg!(windows) { "app.exe" } else { "app" };
        // Note: go build may create "app" or "app.exe" depending on platform
        let app_exists =
            temp_dir.path().join(binary_name).exists() || temp_dir.path().join("app").exists();
        assert!(app_exists, "go build should create binary");
    }
}

// ============================================================================
// Go Mod Tests
// ============================================================================

/// Test: vx go mod init
#[rstest]
#[test]
fn test_go_mod_init() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["go", "mod", "init", "example.com/test"])
        .expect("Failed to run go mod init");

    if is_success(&output) {
        assert!(
            temp_dir.path().join("go.mod").exists(),
            "go mod init should create go.mod"
        );
    }
}

/// Test: vx go mod tidy
#[rstest]
#[test]
fn test_go_mod_tidy() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // First init module
    let init_output = run_vx_in_dir(temp_dir.path(), &["go", "mod", "init", "example.com/test"])
        .expect("Failed to run go mod init");

    if is_success(&init_output) {
        // Create a simple main.go
        std::fs::write(
            temp_dir.path().join("main.go"),
            "package main\n\nfunc main() {}\n",
        )
        .expect("Failed to write main.go");

        let output = run_vx_in_dir(temp_dir.path(), &["go", "mod", "tidy"])
            .expect("Failed to run go mod tidy");

        // go mod tidy should succeed
        assert!(is_success(&output), "go mod tidy should succeed");
    }
}

// ============================================================================
// Go Fmt Tests
// ============================================================================

/// Test: vx go fmt
#[rstest]
#[test]
fn test_go_fmt() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Write unformatted Go code
    let main_go = temp_dir.path().join("main.go");
    std::fs::write(
        &main_go,
        r#"package main
func main(){println("hello")}"#,
    )
    .expect("Failed to write main.go");

    let output =
        run_vx_in_dir(temp_dir.path(), &["go", "fmt", "main.go"]).expect("Failed to run go fmt");

    if is_success(&output) {
        // Read formatted file
        let content = std::fs::read_to_string(&main_go).expect("Failed to read formatted file");
        assert!(
            content.contains("func main() {"),
            "go fmt should format code: {}",
            content
        );
    }
}

// ============================================================================
// Go Vet Tests
// ============================================================================

/// Test: vx go vet
#[rstest]
#[test]
fn test_go_vet() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init module
    let _ = run_vx_in_dir(temp_dir.path(), &["go", "mod", "init", "example.com/test"]);

    // Write valid Go code
    std::fs::write(
        temp_dir.path().join("main.go"),
        r#"package main

func main() {
    println("hello")
}
"#,
    )
    .expect("Failed to write main.go");

    let output = run_vx_in_dir(temp_dir.path(), &["go", "vet", "."]).expect("Failed to run go vet");

    // go vet should succeed for valid code
    if tool_installed("go") {
        assert!(is_success(&output), "go vet should succeed for valid code");
    }
}

// ============================================================================
// Go Test Tests
// ============================================================================

/// Test: vx go test
#[rstest]
#[test]
fn test_go_test() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init module
    let _ = run_vx_in_dir(temp_dir.path(), &["go", "mod", "init", "example.com/test"]);

    // Write a simple test
    std::fs::write(
        temp_dir.path().join("main_test.go"),
        r#"package main

import "testing"

func TestHello(t *testing.T) {
    if 1+1 != 2 {
        t.Error("math is broken")
    }
}
"#,
    )
    .expect("Failed to write test file");

    let output =
        run_vx_in_dir(temp_dir.path(), &["go", "test", "."]).expect("Failed to run go test");

    if is_success(&output) {
        let combined = combined_output(&output);
        assert!(
            combined.contains("ok") || combined.contains("PASS"),
            "go test should pass: {}",
            combined
        );
    }
}

// ============================================================================
// Go Error Handling Tests
// ============================================================================

/// Test: vx go with invalid subcommand
#[rstest]
#[test]
fn test_go_invalid_subcommand() {
    skip_if_no_vx!();

    let output = run_vx(&["go", "invalid-subcommand-xyz"])
        .expect("Failed to run vx go with invalid subcommand");

    if tool_installed("go") {
        assert!(!is_success(&output), "Invalid subcommand should fail");
    }
}

/// Test: vx go run with syntax error
#[rstest]
#[test]
fn test_go_run_syntax_error() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let main_go = temp_dir.path().join("main.go");

    std::fs::write(&main_go, "package main\nfunc main() { invalid syntax }")
        .expect("Failed to write main.go");

    let output = run_vx_in_dir(temp_dir.path(), &["go", "run", "main.go"])
        .expect("Failed to run go run with syntax error");

    if tool_installed("go") {
        assert!(!is_success(&output), "Syntax error should fail");
        let stderr = stderr_str(&output);
        assert!(
            stderr.contains("syntax") || stderr.contains("expected"),
            "Should show syntax error: {}",
            stderr
        );
    }
}

/// Test: vx go build non-existent file
#[rstest]
#[test]
fn test_go_build_nonexistent() {
    skip_if_no_vx!();

    let output = run_vx(&["go", "build", "nonexistent_file.go"])
        .expect("Failed to run go build with missing file");

    if tool_installed("go") {
        assert!(
            !is_success(&output),
            "Building non-existent file should fail"
        );
    }
}
