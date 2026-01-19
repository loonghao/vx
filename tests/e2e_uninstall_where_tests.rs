//! E2E tests for `uninstall` and `where` commands
//!
//! Tests verify:
//! - `vx where <tool>` shows the location of installed tools
//! - `vx uninstall <tool>` removes installed tools
//! - Proper handling of system-installed tools

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

/// E2E test environment with isolated VX_HOME
struct E2ETestEnv {
    home: TempDir,
}

impl E2ETestEnv {
    fn new() -> Self {
        Self {
            home: TempDir::new().expect("Failed to create temp dir"),
        }
    }

    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(vx_binary())
            .args(args)
            .env("VX_HOME", self.home.path())
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

    /// Get the VX_HOME path
    fn home_path(&self) -> &std::path::Path {
        self.home.path()
    }
}

// ============================================================================
// Test: vx where <tool>
// ============================================================================

#[test]
fn test_where_help() {
    let env = E2ETestEnv::new();
    let output = env.run(&["where", "--help"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("where") || stdout.contains("Where") || stdout.contains("location"),
        "Expected help text about where command"
    );
}

#[test]
fn test_where_missing_tool_argument() {
    let env = E2ETestEnv::new();
    let output = env.run(&["where"]);

    // Should fail with usage information
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required") || stderr.contains("TOOL") || stderr.contains("Usage"),
        "Expected usage information, got: {}",
        stderr
    );
}

#[test]
fn test_where_unknown_tool() {
    let env = E2ETestEnv::new();
    let output = env.run(&["where", "nonexistent-tool-xyz-123"]);

    // Should fail with helpful error
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("not found")
            || combined.contains("Unknown")
            || combined.contains("not installed")
            || combined.contains("not supported"),
        "Expected error message, got: {}",
        combined
    );
}

#[test]
fn test_where_tool_not_installed() {
    let env = E2ETestEnv::new();

    // Use a valid tool that's unlikely to be installed in fresh env
    let output = env.run(&["where", "zig"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    // Should indicate the tool is not installed
    // May show system path if installed on the system
    assert!(
        !output.status.success()
            || combined.contains("not installed")
            || combined.contains("system")
            || combined.contains("System")
            || combined.to_lowercase().contains("zig"),
        "Expected not installed message or system path, got: {}",
        combined
    );
}

#[test]
fn test_where_system_installed_tool() {
    let env = E2ETestEnv::new();

    // Test with a commonly system-installed tool
    // On Windows: git is often installed
    // On Unix: python/python3 is often available
    let tool = if cfg!(windows) { "git" } else { "python3" };

    let output = env.run(&["where", tool]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // If the tool is installed on the system, we should see its path
    // If not, we should see a "not found" message
    if output.status.success() {
        // Should show path with "(system)" indicator or actual path
        assert!(
            stdout.contains("system")
                || stdout.contains("System")
                || stdout.contains("/")
                || stdout.contains("\\"),
            "Expected system path in output: {}",
            stdout
        );
    } else {
        // Tool not found is also acceptable
        let combined = format!("{}{}", stdout, stderr);
        assert!(
            combined.contains("not found")
                || combined.contains("not installed")
                || combined.contains("Unknown"),
            "Expected error message, got: {}",
            combined
        );
    }
}

// ============================================================================
// Test: vx uninstall <tool>
// ============================================================================

#[test]
fn test_uninstall_help() {
    let env = E2ETestEnv::new();
    let output = env.run(&["uninstall", "--help"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("uninstall")
            || stdout.contains("Uninstall")
            || stdout.contains("remove")
            || stdout.contains("Remove"),
        "Expected help text about uninstall command"
    );
}

#[test]
fn test_uninstall_missing_tool_argument() {
    let env = E2ETestEnv::new();
    let output = env.run(&["uninstall"]);

    // Should fail with usage information
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required") || stderr.contains("TOOL") || stderr.contains("Usage"),
        "Expected usage information, got: {}",
        stderr
    );
}

#[test]
fn test_uninstall_unknown_tool() {
    let env = E2ETestEnv::new();
    let output = env.run(&["uninstall", "nonexistent-tool-xyz-123"]);

    // Should fail with helpful error
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("not found")
            || combined.contains("Unknown")
            || combined.contains("not installed")
            || combined.contains("not supported"),
        "Expected error message, got: {}",
        combined
    );
}

#[test]
fn test_uninstall_tool_not_installed() {
    let env = E2ETestEnv::new();

    // Try to uninstall a valid tool that's not installed
    let output = env.run(&["uninstall", "zig"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    // Should indicate no versions are installed
    assert!(
        combined.contains("not installed")
            || combined.contains("No version")
            || combined.contains("nothing to uninstall")
            || combined.to_lowercase().contains("no version"),
        "Expected 'not installed' message, got: {}",
        combined
    );
}

#[test]
fn test_uninstall_system_tool_gives_guidance() {
    let env = E2ETestEnv::new();

    // ImageMagick is a good test case - often system-installed
    // Even if not installed, the error message should be helpful
    let output = env.run(&["uninstall", "magick"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    // Should either:
    // 1. Say "no versions installed" if not installed anywhere
    // 2. Give package manager guidance if system-installed
    assert!(
        combined.contains("not installed")
            || combined.contains("No version")
            || combined.contains("system")
            || combined.contains("package manager")
            || combined.contains("choco")
            || combined.contains("brew")
            || combined.contains("apt"),
        "Expected helpful message about uninstalling, got: {}",
        combined
    );
}

// ============================================================================
// Test: Install and Uninstall workflow
// ============================================================================

#[test]
#[ignore] // This test downloads files, run manually or in CI
fn test_install_where_uninstall_workflow() {
    let env = E2ETestEnv::new();

    // 1. Install a small, quick tool
    let install_output = env.run(&["install", "zig@0.15.2"]);
    if !install_output.status.success() {
        let stderr = String::from_utf8_lossy(&install_output.stderr);
        // Network errors are acceptable in CI
        if stderr.contains("network") || stderr.contains("timeout") || stderr.contains("rate limit")
        {
            return; // Skip test if network issues
        }
        panic!("Install failed: {}", stderr);
    }

    // 2. Verify with "where" command
    let where_output = env.run(&["where", "zig"]);
    assert!(
        where_output.status.success(),
        "where command should succeed after install"
    );

    let stdout = String::from_utf8_lossy(&where_output.stdout);
    assert!(
        stdout.contains(env.home_path().to_str().unwrap()) || stdout.contains("zig"),
        "where should show installed path"
    );

    // 3. List should show the installed version
    let list_output = env.run(&["list", "zig"]);
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        list_stdout.contains("0.15.2") || list_stdout.contains("zig"),
        "list should show installed version"
    );

    // 4. Uninstall
    let uninstall_output = env.run(&["uninstall", "zig@0.15.2"]);
    assert!(
        uninstall_output.status.success(),
        "uninstall should succeed"
    );

    // 5. Verify it's gone
    let where_after = env.run(&["where", "zig"]);
    let after_combined = format!(
        "{}{}",
        String::from_utf8_lossy(&where_after.stdout),
        String::from_utf8_lossy(&where_after.stderr)
    );
    assert!(
        !where_after.status.success() || after_combined.contains("not installed"),
        "Tool should be uninstalled"
    );
}

// ============================================================================
// Test: Multiple providers
// ============================================================================

#[test]
fn test_where_multiple_tools() {
    let env = E2ETestEnv::new();

    // Test where command for various tools
    let tools = ["node", "go", "uv", "zig", "rust"];

    for tool in &tools {
        let output = env.run(&["where", tool]);
        // Should not crash - either find it or report not found
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Verify we get a sensible response
        assert!(
            output.status.success()
                || stderr.contains("not found")
                || stderr.contains("not installed")
                || stdout.contains("not installed"),
            "Tool '{}' should have proper response",
            tool
        );
    }
}

#[test]
fn test_uninstall_multiple_tools_not_installed() {
    let env = E2ETestEnv::new();

    // Test uninstall command for various tools (none installed)
    let tools = ["node", "go", "uv"];

    for tool in &tools {
        let output = env.run(&["uninstall", tool]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}{}", stdout, stderr);

        // Should report not installed (not crash)
        assert!(
            combined.contains("not installed")
                || combined.contains("No version")
                || combined.to_lowercase().contains("no version"),
            "Tool '{}' should report not installed, got: {}",
            tool,
            combined
        );
    }
}

// ============================================================================
// Test: ImageMagick specific (system installation detection)
// ============================================================================

#[test]
fn test_imagemagick_where_detects_system() {
    let env = E2ETestEnv::new();

    let output = env.run(&["where", "magick"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // If ImageMagick is installed on the system, should show "(system)"
    // If not installed, should say not found
    if output.status.success() {
        assert!(
            combined.contains("system")
                || combined.contains("magick")
                || combined.contains("ImageMagick"),
            "Expected system path indicator for ImageMagick: {}",
            combined
        );
    } else {
        assert!(
            combined.contains("not found")
                || combined.contains("not installed")
                || combined.contains("No version"),
            "Expected 'not found' message for ImageMagick: {}",
            combined
        );
    }
}

#[test]
fn test_imagemagick_install_unsupported_platform() {
    let env = E2ETestEnv::new();

    // On Windows/macOS, ImageMagick direct download is not supported
    // Should give helpful error message with package manager instructions
    if cfg!(windows) || cfg!(target_os = "macos") {
        let output = env.run(&["install", "magick@latest"]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}{}", stdout, stderr);

        // Should fail with helpful message
        assert!(
            !output.status.success(),
            "ImageMagick install should fail on Windows/macOS without direct download"
        );

        // Should mention package manager
        assert!(
            combined.contains("choco")
                || combined.contains("scoop")
                || combined.contains("brew")
                || combined.contains("package manager")
                || combined.contains("system package"),
            "Expected package manager guidance, got: {}",
            combined
        );
    }
}

// ============================================================================
// Test: Alias handling
// ============================================================================

#[test]
fn test_where_alias_imagemagick() {
    let env = E2ETestEnv::new();

    // "imagemagick" is an alias for "magick"
    let output_alias = env.run(&["where", "imagemagick"]);
    let output_primary = env.run(&["where", "magick"]);

    // Both should give same result (or both fail)
    assert_eq!(
        output_alias.status.success(),
        output_primary.status.success(),
        "Alias 'imagemagick' should behave same as 'magick'"
    );
}

#[test]
fn test_where_alias_nodejs() {
    let env = E2ETestEnv::new();

    // "nodejs" is an alias for "node"
    let output_alias = env.run(&["where", "nodejs"]);
    let output_primary = env.run(&["where", "node"]);

    // Both should give same result
    assert_eq!(
        output_alias.status.success(),
        output_primary.status.success(),
        "Alias 'nodejs' should behave same as 'node'"
    );
}
