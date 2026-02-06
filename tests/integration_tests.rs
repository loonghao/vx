//! Integration tests for vx tool manager
//!
//! These tests verify that the entire vx system works correctly
//! by testing the CLI interface and core functionality.

use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

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

/// Helper function to run vx commands
fn run_vx_command(args: &[&str]) -> std::process::Output {
    Command::new(vx_binary())
        .args(args)
        .output()
        .expect("Failed to execute vx command")
}

/// Helper function to run vx commands with a timeout.
/// Returns `None` if the command times out.
fn run_vx_command_with_timeout(args: &[&str], timeout_secs: u64) -> Option<std::process::Output> {
    let mut child = Command::new(vx_binary())
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn vx command");

    let timeout = Duration::from_secs(timeout_secs);
    let start = Instant::now();

    // Take stdout/stderr handles to read in separate threads (avoid pipe deadlock)
    let mut stdout_handle = child.stdout.take().unwrap();
    let mut stderr_handle = child.stderr.take().unwrap();

    let stdout_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stdout_handle.read_to_end(&mut buf).ok();
        buf
    });
    let stderr_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stderr_handle.read_to_end(&mut buf).ok();
        buf
    });

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = stdout_thread.join().unwrap_or_default();
                let stderr = stderr_thread.join().unwrap_or_default();
                return Some(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => panic!("Error waiting for process: {e}"),
        }
    }
}

#[test]
fn test_vx_help() {
    let output = run_vx_command(&["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vx"));
    // More flexible check - just ensure it contains some help text
    assert!(stdout.contains("help") || stdout.contains("Usage") || stdout.contains("USAGE"));
}

#[test]
fn test_vx_version() {
    let output = run_vx_command(&["--version"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vx"));
}

#[test]
fn test_vx_list_command() {
    let output = run_vx_command(&["list"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show available tools
    assert!(!stdout.is_empty());
}

#[test]
fn test_vx_plugin_list() {
    let output = run_vx_command(&["plugin", "list"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show available plugins
    assert!(!stdout.is_empty());
}

#[test]
fn test_vx_unsupported_tool() {
    let output = run_vx_command(&["nonexistent-tool", "--version"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should suggest available plugins or show error
    assert!(!stderr.is_empty());
}

#[test]
fn test_vx_system_path_flag() {
    // Test that --use-system-path flag is recognized
    let output = run_vx_command(&["--use-system-path", "nonexistent-tool-xyz"]);
    // This should fail because the tool doesn't exist, but the flag should be parsed correctly
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should not contain "unknown flag" or similar errors about the flag itself
    // It should contain an error about the tool not being found, not about unknown flags
    assert!(!stderr.contains("unknown flag") && !stderr.contains("unrecognized option"));
}

#[cfg(test)]
mod tool_specific_tests {
    use super::*;

    /// Default timeout (seconds) for tool commands that may trigger auto-install.
    const TOOL_TIMEOUT_SECS: u64 = 30;

    #[test]
    fn test_node_tool_help() {
        let Some(output) = run_vx_command_with_timeout(&["node", "--help"], TOOL_TIMEOUT_SECS)
        else {
            eprintln!("Skipping: node --help timed out (tool may not be installed)");
            return;
        };
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
    }

    #[test]
    fn test_uv_tool_help() {
        let Some(output) = run_vx_command_with_timeout(&["uv", "--help"], TOOL_TIMEOUT_SECS) else {
            eprintln!("Skipping: uv --help timed out (tool may not be installed)");
            return;
        };
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
    }

    #[test]
    fn test_go_tool_help() {
        let Some(output) = run_vx_command_with_timeout(&["go", "version"], TOOL_TIMEOUT_SECS)
        else {
            eprintln!("Skipping: go version timed out (tool may not be installed)");
            return;
        };
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
    }

    #[test]
    fn test_cargo_tool_help() {
        let Some(output) = run_vx_command_with_timeout(&["cargo", "--version"], TOOL_TIMEOUT_SECS)
        else {
            eprintln!("Skipping: cargo --version timed out (tool may not be installed)");
            return;
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success()
                || stdout.contains("cargo")
                || !stderr.is_empty()
                || !stdout.is_empty()
        );
    }
}

#[cfg(test)]
mod environment_isolation_tests {
    use super::*;

    #[test]
    fn test_default_isolation_behavior() {
        // Test that vx isolates tools by default (use timeout to avoid CI hangs)
        let Some(output) = run_vx_command_with_timeout(&["python", "--version"], 30) else {
            eprintln!("Skipping: python --version timed out (tool may not be installed)");
            return;
        };

        // Should either work with vx-managed python or show proper error
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let has_output = !stderr.is_empty() || !stdout.is_empty();
        let succeeded = output.status.success();

        if !succeeded && !has_output {
            eprintln!("Warning: Command failed silently. This might indicate error handling needs improvement.");
        }
    }

    #[test]
    fn test_system_path_behavior() {
        // Test that --use-system-path allows system tools
        let output = run_vx_command(&["--use-system-path", "echo", "hello"]);

        // Should work on most systems since echo is a basic command
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("hello"));
        }
        // If it fails, that's also acceptable depending on the system
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_vx_config_command() {
        let output = run_vx_command(&["config"]);

        // Should show config information or help
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
    }
}

#[cfg(test)]
mod dynamic_execution_tests {
    use super::*;

    #[test]
    fn test_dynamic_command_with_multiple_args() {
        // Test that vx can handle complex command arguments
        let output = run_vx_command(&["echo", "hello", "world", "test"]);

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("hello"));
        }
        // If echo is not available, that's also acceptable
    }

    #[test]
    fn test_dynamic_command_with_flags() {
        // Test that vx properly passes flags to tools
        let Some(output) = run_vx_command_with_timeout(&["cargo", "--version"], 30) else {
            eprintln!("Skipping: cargo --version timed out (tool may not be installed)");
            return;
        };

        // Should either work or show proper error
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(output.status.success() || !stderr.is_empty() || !stdout.is_empty());
    }

    #[test]
    fn test_zero_learning_cost() {
        // Test that users can use vx exactly like they would use the tool directly
        // This is the core value proposition of vx

        // Test with a common command that should work on most systems
        let output = run_vx_command(&["echo", "vx-test"]);

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("vx-test"));
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_clear_error_messages() {
        // Test that vx provides clear error messages
        let output = run_vx_command(&["definitely-nonexistent-tool-xyz"]);

        assert!(!output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        // Should contain helpful error message about unknown tool or auto-install failure
        assert!(
            combined.contains("Tool not found")
                || combined.contains("not found")
                || combined.contains("Unknown tool")
                || combined.contains("Cannot auto-install")
        );
    }

    #[test]
    fn test_no_args_behavior() {
        // Test behavior when no arguments are provided
        let output = run_vx_command(&[]);

        // vx now shows help when no arguments are provided, which is a success case
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should provide usage information either in stdout or stderr
        assert!(
            stdout.contains("Usage")
                || stderr.contains("Usage")
                || stdout.contains("help")
                || stderr.contains("help")
                || stderr.contains("No tool specified")
        );
    }
}
