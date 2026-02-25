//! E2E tests for package execution syntax
//!
//! These tests verify the `ecosystem:package` syntax for all supported ecosystems,
//! including the newly added oneshot runners (pipx, dlx, deno, dotnet-tool, jbang)
//! and the shell execution syntax (`runtime::shell`).
//!
//! ## Supported Syntax
//!
//! ### Package Execution (RFC 0027)
//! ```
//! vx <ecosystem>[@runtime_version]:<package>[@version][::executable] [args...]
//! ```
//!
//! ### Shell Execution
//! ```
//! vx <runtime>[::shell_name]
//! ```
//!
//! ## Supported Ecosystems
//!
//! | Ecosystem | Aliases | Description |
//! |-----------|---------|-------------|
//! | `npm`     | `node`, `npx` | Node.js packages via npm |
//! | `bun`     | `bunx` | Node.js packages via bun |
//! | `pnpm`    | -      | Node.js packages via pnpm |
//! | `yarn`    | -      | Node.js packages via yarn |
//! | `dlx`     | -      | pnpm dlx oneshot runner |
//! | `pip`     | `python`, `pypi` | Python packages via pip |
//! | `uv`      | -      | Python packages via uv |
//! | `uvx`     | -      | Python CLI tools via uvx |
//! | `pipx`    | -      | Python CLI tools via pipx |
//! | `deno`    | -      | npm/JSR packages via deno |
//! | `dotnet-tool` | `dotnet` | .NET tools via dotnet tool install |
//! | `jbang`   | `java` | Java tools via jbang |
//! | `cargo`   | `rust`, `crates` | Rust packages via cargo |
//! | `go`      | `golang` | Go packages via go install |
//! | `gem`     | `ruby`, `rubygems` | Ruby gems |

use std::env;
use std::path::PathBuf;
use std::process::Command;

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

/// Run vx with the given args and return (success, stdout, stderr)
fn run_vx(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(vx_binary())
        .args(args)
        .output()
        .expect("Failed to execute vx command");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

// ============================================
// Package Request Syntax Recognition Tests
// ============================================

/// Test that `ecosystem:package` syntax is recognized as a package request
/// These tests verify the parser recognizes the syntax without actually installing
#[test]
fn test_package_syntax_npm_recognized() {
    // npm:package syntax should be recognized (may fail due to no network, but not "unknown command")
    let (_, stdout, stderr) = run_vx(&["npm:typescript", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    // Should NOT say "unknown command" or "not a valid runtime"
    // It should either install/run or say "not installed"
    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "npm:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_npx_recognized() {
    // npx:package syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["npx:cowsay", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "npx:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_bunx_recognized() {
    // bunx:package syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["bunx:cowsay", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "bunx:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_pip_recognized() {
    // pip:package syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["pip:black", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "pip:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_uvx_recognized() {
    // uvx:package syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["uvx:ruff", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "uvx:package syntax not recognized: {}",
        combined
    );
}

// ============================================
// New Oneshot Runner Syntax Tests
// ============================================

#[test]
fn test_package_syntax_pipx_recognized() {
    // pipx:package syntax should be recognized (Python oneshot runner)
    let (_, stdout, stderr) = run_vx(&["pipx:cowsay", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    // Should be recognized as a package request, not an unknown command
    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "pipx:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_dlx_recognized() {
    // dlx:package syntax should be recognized (pnpm dlx oneshot runner)
    let (_, stdout, stderr) = run_vx(&["dlx:create-react-app", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "dlx:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_deno_recognized() {
    // deno:package syntax should be recognized (Deno oneshot runner)
    let (_, stdout, stderr) = run_vx(&["deno:cowsay", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "deno:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_dotnet_tool_recognized() {
    // dotnet-tool:package syntax should be recognized (.NET tool runner)
    let (_, stdout, stderr) = run_vx(&["dotnet-tool:dotnet-script", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "dotnet-tool:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_dotnet_alias_recognized() {
    // dotnet:package syntax (alias for dotnet-tool) should be recognized
    let (_, stdout, stderr) = run_vx(&["dotnet:dotnet-script", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "dotnet:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_jbang_recognized() {
    // jbang:package syntax should be recognized (Java oneshot runner)
    let (_, stdout, stderr) = run_vx(&["jbang:picocli", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "jbang:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_java_alias_recognized() {
    // java:package syntax (alias for jbang) should be recognized
    let (_, stdout, stderr) = run_vx(&["java:picocli", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "java:package syntax not recognized: {}",
        combined
    );
}

// ============================================
// Existing Ecosystem Syntax Tests
// ============================================

#[test]
fn test_package_syntax_cargo_recognized() {
    // cargo:package syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["cargo:ripgrep::rg", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "cargo:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_go_recognized() {
    // go:package syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["go:golang.org/x/tools/gopls::gopls", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "go:package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_gem_recognized() {
    // gem:package syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["gem:bundler", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "gem:package syntax not recognized: {}",
        combined
    );
}

// ============================================
// Package Syntax with Version Tests
// ============================================

#[test]
fn test_package_syntax_with_version() {
    // ecosystem:package@version syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["npm:typescript@5.3", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "npm:package@version syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_with_executable() {
    // ecosystem:package::executable syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["npm:typescript::tsc", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "npm:package::executable syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_full_spec() {
    // Full syntax: ecosystem@runtime_version:package@version::executable
    let (_, stdout, stderr) = run_vx(&["npm@20:typescript@5.3::tsc", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "Full package syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_package_syntax_scoped_npm() {
    // Scoped npm package: npm:@scope/package
    let (_, stdout, stderr) = run_vx(&["npm:@biomejs/biome::biome", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        !combined.contains("unknown command")
            && !combined.contains("not a valid runtime")
            && !combined.contains("No such subcommand"),
        "Scoped npm package syntax not recognized: {}",
        combined
    );
}

// ============================================
// Shell Execution Syntax Tests
// ============================================

#[test]
fn test_shell_syntax_recognized() {
    // runtime::shell syntax should be recognized
    // We test with a non-existent shell to verify the syntax is parsed correctly
    // (it will fail because the shell doesn't exist, but not with "unknown command")
    let (_, stdout, stderr) = run_vx(&["git::nonexistent-shell-xyz"]);
    let combined = format!("{}{}", stdout, stderr);

    // Should NOT say "unknown command" - it should try to launch the shell
    // and fail because the shell doesn't exist
    assert!(
        !combined.contains("unknown command") && !combined.contains("No such subcommand"),
        "Shell syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_shell_syntax_git_bash_recognized() {
    // git::git-bash syntax should be recognized
    // On non-Windows or without git-bash installed, it will fail gracefully
    let (_, stdout, stderr) = run_vx(&["git::git-bash", "--help"]);
    let combined = format!("{}{}", stdout, stderr);

    // Should NOT say "unknown command"
    assert!(
        !combined.contains("unknown command") && !combined.contains("No such subcommand"),
        "git::git-bash syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_shell_syntax_node_cmd() {
    // node::cmd syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["node::cmd", "/c", "echo", "test"]);
    let combined = format!("{}{}", stdout, stderr);

    // Should NOT say "unknown command"
    assert!(
        !combined.contains("unknown command") && !combined.contains("No such subcommand"),
        "node::cmd syntax not recognized: {}",
        combined
    );
}

#[test]
fn test_shell_syntax_go_powershell() {
    // go::powershell syntax should be recognized
    let (_, stdout, stderr) = run_vx(&["go::powershell", "-Command", "echo test"]);
    let combined = format!("{}{}", stdout, stderr);

    // Should NOT say "unknown command"
    assert!(
        !combined.contains("unknown command") && !combined.contains("No such subcommand"),
        "go::powershell syntax not recognized: {}",
        combined
    );
}

// ============================================
// Help and Version Tests
// ============================================

#[test]
fn test_help_shows_package_syntax() {
    // vx --help should mention the ecosystem:package syntax
    let (success, stdout, _stderr) = run_vx(&["--help"]);

    assert!(success, "vx --help should succeed");
    // Help should mention the package execution syntax
    assert!(
        stdout.contains("ecosystem")
            || stdout.contains("package")
            || stdout.contains("npm:")
            || stdout.contains("pip:")
            || stdout.contains("cargo:"),
        "Help should mention package execution syntax: {}",
        stdout
    );
}

#[test]
fn test_help_shows_shell_syntax() {
    // vx --help should mention the shell execution syntax
    let (success, stdout, _stderr) = run_vx(&["--help"]);

    assert!(success, "vx --help should succeed");
    // Help should mention the shell syntax
    assert!(
        stdout.contains("::")
            || stdout.contains("shell")
            || stdout.contains("git-bash")
            || stdout.contains("powershell"),
        "Help should mention shell execution syntax: {}",
        stdout
    );
}

// ============================================
// Ecosystem Alias Tests
// ============================================

#[test]
fn test_ecosystem_aliases_recognized() {
    // Test that ecosystem aliases are recognized
    let aliases = [
        ("node:cowsay", "node alias for npm"),
        ("python:black", "python alias for pip"),
        ("pypi:black", "pypi alias for pip"),
        ("rust:ripgrep::rg", "rust alias for cargo"),
        ("crates:ripgrep::rg", "crates alias for cargo"),
        ("golang:gopls", "golang alias for go"),
        ("ruby:bundler", "ruby alias for gem"),
        ("rubygems:bundler", "rubygems alias for gem"),
    ];

    for (spec, description) in &aliases {
        let (_, stdout, stderr) = run_vx(&[spec, "--version"]);
        let combined = format!("{}{}", stdout, stderr);

        assert!(
            !combined.contains("unknown command")
                && !combined.contains("not a valid runtime")
                && !combined.contains("No such subcommand"),
            "{} not recognized: {}",
            description,
            combined
        );
    }
}

// ============================================
// Error Handling Tests
// ============================================

#[test]
fn test_invalid_ecosystem_gives_helpful_error() {
    // An invalid ecosystem should give a helpful error message
    let (success, stdout, stderr) = run_vx(&["invalid-ecosystem-xyz:some-package"]);
    let combined = format!("{}{}", stdout, stderr);

    // Should fail
    assert!(!success, "Invalid ecosystem should fail");

    // Should give a helpful error
    assert!(
        combined.contains("Unsupported ecosystem")
            || combined.contains("not supported")
            || combined.contains("invalid")
            || combined.contains("error"),
        "Should give helpful error for invalid ecosystem: {}",
        combined
    );
}

#[test]
fn test_package_not_installed_gives_helpful_message() {
    // When a package is not installed, should give a helpful message
    // (not crash or give cryptic error)
    let (_, stdout, stderr) = run_vx(&["npm:definitely-not-a-real-package-xyz-12345", "--version"]);
    let combined = format!("{}{}", stdout, stderr);

    // Should give some kind of message (installing, not found, etc.)
    assert!(
        !combined.is_empty(),
        "Should give some output for missing package"
    );
}
