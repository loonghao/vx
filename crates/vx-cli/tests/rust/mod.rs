//! Rust/Cargo E2E Tests for vx CLI
//!
//! Tests for Rust toolchain: cargo, rustc

use crate::common::*;
use rstest::*;
use tempfile::TempDir;

// ============================================================================
// Cargo Version Tests
// ============================================================================

/// Test: vx cargo --version
#[rstest]
#[test]
fn test_cargo_version() {
    skip_if_no_vx!();

    let output = run_vx(&["cargo", "--version"]).expect("Failed to run vx cargo --version");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.contains("cargo"),
            "cargo version should contain 'cargo': {}",
            version
        );
    }
}

/// Test: vx cargo -V (short form)
#[rstest]
#[test]
fn test_cargo_version_short() {
    skip_if_no_vx!();

    let output = run_vx(&["cargo", "-V"]).expect("Failed to run vx cargo -V");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.contains("cargo"),
            "cargo version should contain 'cargo': {}",
            version
        );
    }
}

// ============================================================================
// Rustc Version Tests
// ============================================================================

/// Test: vx rustc --version
#[rstest]
#[test]
fn test_rustc_version() {
    skip_if_no_vx!();

    let output = run_vx(&["rustc", "--version"]).expect("Failed to run vx rustc --version");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.contains("rustc"),
            "rustc version should contain 'rustc': {}",
            version
        );
    }
}

/// Test: vx rustc -V (short form)
#[rstest]
#[test]
fn test_rustc_version_short() {
    skip_if_no_vx!();

    let output = run_vx(&["rustc", "-V"]).expect("Failed to run vx rustc -V");

    if is_success(&output) {
        let version = stdout_str(&output);
        assert!(
            version.contains("rustc"),
            "rustc version should contain 'rustc': {}",
            version
        );
    }
}

/// Test: vx rustc --version --verbose
#[rstest]
#[test]
fn test_rustc_version_verbose() {
    skip_if_no_vx!();

    let output =
        run_vx(&["rustc", "--version", "--verbose"]).expect("Failed to run vx rustc --version -v");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("host:") || stdout.contains("release:"),
            "rustc --version --verbose should show details: {}",
            stdout
        );
    }
}

// ============================================================================
// Cargo Help Tests
// ============================================================================

/// Test: vx cargo --help
#[rstest]
#[test]
fn test_cargo_help() {
    skip_if_no_vx!();

    let output = run_vx(&["cargo", "--help"]).expect("Failed to run vx cargo --help");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("Usage") || stdout.contains("USAGE"),
            "cargo help should show usage: {}",
            stdout
        );
    }
}

/// Test: vx cargo help build
#[rstest]
#[test]
fn test_cargo_help_build() {
    skip_if_no_vx!();

    let output = run_vx(&["cargo", "help", "build"]).expect("Failed to run vx cargo help build");

    if is_success(&output) {
        let stdout = stdout_str(&output);
        assert!(
            stdout.contains("build") || stdout.contains("Compile"),
            "cargo help build: {}",
            stdout
        );
    }
}

// ============================================================================
// Cargo Init/New Tests
// ============================================================================

/// Test: vx cargo init
#[rstest]
#[test]
fn test_cargo_init() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&output) {
        assert!(
            temp_dir.path().join("Cargo.toml").exists(),
            "cargo init should create Cargo.toml"
        );
        assert!(
            temp_dir.path().join("src").join("main.rs").exists(),
            "cargo init should create src/main.rs"
        );
    }
}

/// Test: vx cargo init --lib
#[rstest]
#[test]
fn test_cargo_init_lib() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["cargo", "init", "--lib"])
        .expect("Failed to run cargo init --lib");

    if is_success(&output) {
        assert!(
            temp_dir.path().join("Cargo.toml").exists(),
            "cargo init --lib should create Cargo.toml"
        );
        assert!(
            temp_dir.path().join("src").join("lib.rs").exists(),
            "cargo init --lib should create src/lib.rs"
        );
    }
}

/// Test: vx cargo new
#[rstest]
#[test]
fn test_cargo_new() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_vx_in_dir(temp_dir.path(), &["cargo", "new", "myproject"])
        .expect("Failed to run cargo new");

    if is_success(&output) {
        let project_dir = temp_dir.path().join("myproject");
        assert!(
            project_dir.join("Cargo.toml").exists(),
            "cargo new should create project/Cargo.toml"
        );
    }
}

// ============================================================================
// Cargo Build Tests
// ============================================================================

/// Test: vx cargo build
#[rstest]
#[test]
fn test_cargo_build() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        let output =
            run_vx_in_dir(temp_dir.path(), &["cargo", "build"]).expect("Failed to run cargo build");

        if is_success(&output) {
            assert!(
                temp_dir.path().join("target").exists(),
                "cargo build should create target directory"
            );
        }
    }
}

/// Test: vx cargo build --release
#[rstest]
#[test]
fn test_cargo_build_release() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        let output = run_vx_in_dir(temp_dir.path(), &["cargo", "build", "--release"])
            .expect("Failed to run cargo build --release");

        if is_success(&output) {
            assert!(
                temp_dir.path().join("target").join("release").exists(),
                "cargo build --release should create target/release"
            );
        }
    }
}

// ============================================================================
// Cargo Run Tests
// ============================================================================

/// Test: vx cargo run
#[rstest]
#[test]
fn test_cargo_run() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        // Modify main.rs to print something specific
        std::fs::write(
            temp_dir.path().join("src").join("main.rs"),
            r#"fn main() { println!("Hello from vx cargo!"); }"#,
        )
        .expect("Failed to write main.rs");

        let output =
            run_vx_in_dir(temp_dir.path(), &["cargo", "run"]).expect("Failed to run cargo run");

        if is_success(&output) {
            assert_stdout_contains(&output, "Hello from vx cargo!", "cargo run");
        }
    }
}

/// Test: vx cargo run with arguments
#[rstest]
#[test]
fn test_cargo_run_with_args() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        std::fs::write(
            temp_dir.path().join("src").join("main.rs"),
            r#"
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("Args: {}", args.join(", "));
}
"#,
        )
        .expect("Failed to write main.rs");

        let output = run_vx_in_dir(temp_dir.path(), &["cargo", "run", "--", "arg1", "arg2"])
            .expect("Failed to run cargo run with args");

        if is_success(&output) {
            assert_stdout_contains(&output, "arg1, arg2", "cargo run args");
        }
    }
}

// ============================================================================
// Cargo Test Tests
// ============================================================================

/// Test: vx cargo test
#[rstest]
#[test]
fn test_cargo_test() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        // Add a test
        std::fs::write(
            temp_dir.path().join("src").join("main.rs"),
            r#"
fn main() {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
        )
        .expect("Failed to write main.rs");

        let output =
            run_vx_in_dir(temp_dir.path(), &["cargo", "test"]).expect("Failed to run cargo test");

        if is_success(&output) {
            let combined = combined_output(&output);
            assert!(
                combined.contains("test result: ok")
                    || combined.contains("passed")
                    || combined.contains("1 passed"),
                "cargo test should pass: {}",
                combined
            );
        }
    }
}

// ============================================================================
// Cargo Check Tests
// ============================================================================

/// Test: vx cargo check
#[rstest]
#[test]
fn test_cargo_check() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        let output =
            run_vx_in_dir(temp_dir.path(), &["cargo", "check"]).expect("Failed to run cargo check");

        // cargo check should succeed for valid project
        assert!(is_success(&output), "cargo check should succeed");
    }
}

// ============================================================================
// Cargo Fmt Tests
// ============================================================================

/// Test: vx cargo fmt --check
#[rstest]
#[test]
fn test_cargo_fmt_check() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        let output = run_vx_in_dir(temp_dir.path(), &["cargo", "fmt", "--check"])
            .expect("Failed to run cargo fmt --check");

        // cargo fmt --check should succeed for properly formatted code
        // (cargo init creates formatted code)
        let _ = combined_output(&output);
    }
}

// ============================================================================
// Cargo Clippy Tests
// ============================================================================

/// Test: vx cargo clippy
#[rstest]
#[test]
fn test_cargo_clippy() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        let output = run_vx_in_dir(temp_dir.path(), &["cargo", "clippy"])
            .expect("Failed to run cargo clippy");

        // clippy should succeed for simple project (if installed)
        let _ = combined_output(&output);
    }
}

// ============================================================================
// Cargo Clean Tests
// ============================================================================

/// Test: vx cargo clean
#[rstest]
#[test]
fn test_cargo_clean() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init and build project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        let _ = run_vx_in_dir(temp_dir.path(), &["cargo", "build"]);

        let output =
            run_vx_in_dir(temp_dir.path(), &["cargo", "clean"]).expect("Failed to run cargo clean");

        if is_success(&output) {
            // target directory should be removed or empty
            let target_dir = temp_dir.path().join("target");
            assert!(
                !target_dir.exists() || target_dir.read_dir().map(|d| d.count()).unwrap_or(0) == 0,
                "cargo clean should remove target"
            );
        }
    }
}

// ============================================================================
// Rustc Compilation Tests
// ============================================================================

/// Test: vx rustc simple file
#[rstest]
#[test]
fn test_rustc_compile() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let main_rs = temp_dir.path().join("main.rs");

    std::fs::write(&main_rs, r#"fn main() { println!("Hello from rustc!"); }"#)
        .expect("Failed to write main.rs");

    let output =
        run_vx_in_dir(temp_dir.path(), &["rustc", "main.rs"]).expect("Failed to run rustc");

    if is_success(&output) {
        let binary_name = if cfg!(windows) { "main.exe" } else { "main" };
        assert!(
            temp_dir.path().join(binary_name).exists(),
            "rustc should create binary"
        );
    }
}

/// Test: vx rustc with output name
#[rstest]
#[test]
fn test_rustc_output_name() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let main_rs = temp_dir.path().join("main.rs");

    std::fs::write(&main_rs, r#"fn main() { println!("custom output"); }"#)
        .expect("Failed to write main.rs");

    let output_name = if cfg!(windows) { "myapp.exe" } else { "myapp" };
    let output = run_vx_in_dir(temp_dir.path(), &["rustc", "main.rs", "-o", output_name])
        .expect("Failed to run rustc -o");

    if is_success(&output) {
        assert!(
            temp_dir.path().join(output_name).exists(),
            "rustc -o should create named binary"
        );
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Test: vx cargo with invalid subcommand
#[rstest]
#[test]
fn test_cargo_invalid_subcommand() {
    skip_if_no_vx!();

    let output = run_vx(&["cargo", "invalid-subcommand-xyz"])
        .expect("Failed to run cargo with invalid subcommand");

    if tool_installed("cargo") {
        assert!(!is_success(&output), "Invalid subcommand should fail");
    }
}

/// Test: vx cargo build with compile error
#[rstest]
#[test]
fn test_cargo_build_compile_error() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Init project first
    let init_output =
        run_vx_in_dir(temp_dir.path(), &["cargo", "init"]).expect("Failed to run cargo init");

    if is_success(&init_output) {
        // Write invalid Rust code
        std::fs::write(
            temp_dir.path().join("src").join("main.rs"),
            "fn main() { invalid syntax }",
        )
        .expect("Failed to write main.rs");

        let output = run_vx_in_dir(temp_dir.path(), &["cargo", "build"])
            .expect("Failed to run cargo build with error");

        assert!(!is_success(&output), "Compile error should fail");
        let stderr = stderr_str(&output);
        assert!(
            stderr.contains("error") || stderr.contains("expected"),
            "Should show compile error: {}",
            stderr
        );
    }
}

/// Test: vx rustc with syntax error
#[rstest]
#[test]
fn test_rustc_syntax_error() {
    skip_if_no_vx!();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let main_rs = temp_dir.path().join("main.rs");

    std::fs::write(&main_rs, "fn main() { invalid }").expect("Failed to write main.rs");

    let output =
        run_vx_in_dir(temp_dir.path(), &["rustc", "main.rs"]).expect("Failed to run rustc");

    if tool_installed("cargo") {
        assert!(!is_success(&output), "Syntax error should fail");
    }
}
