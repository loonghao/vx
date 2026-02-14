//! Cross-platform command execution utilities
//!
//! This module provides utilities for executing commands in a cross-platform manner,
//! handling Windows-specific issues like `.cmd`/`.bat` files with spaces in paths.
//!
//! ## Windows .cmd/.bat Handling
//!
//! On Windows, `.cmd` and `.bat` files need special handling because:
//! 1. They must be executed via `cmd.exe /c`
//! 2. `cmd.exe` has non-standard quoting rules (doesn't follow `CommandLineToArgvW`)
//! 3. Paths with spaces require careful quoting
//!
//! ## Usage
//!
//! All providers should use `spawn_command` instead of directly using
//! `tokio::process::Command::new()` when executing external programs that might
//! be `.cmd` or `.bat` files (especially on Windows).
//!
//! ```rust,ignore
//! use vx_core::command::spawn_command;
//! use std::path::Path;
//!
//! let output = spawn_command(Path::new("C:\\Program Files\\nodejs\\npm.cmd"), &["--version"])
//!     .await?;
//! ```

use std::path::Path;
use std::process::Output;
use tokio::process::Command;

/// Error type for command execution
#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command failed with exit code {exit_code}: {message}")]
    Failed { exit_code: i32, message: String },
}

/// Result type for command operations
pub type CommandResult<T> = Result<T, CommandError>;

/// Spawn a command with proper cross-platform handling
///
/// This function handles:
/// - Windows `.cmd`/`.bat` files via `cmd.exe /c` with proper quoting
/// - Unix executables directly
///
/// # Arguments
///
/// * `executable` - Path to the executable
/// * `args` - Command arguments
///
/// # Returns
///
/// The command output (stdout, stderr, exit status)
///
/// # Example
///
/// ```rust,ignore
/// use vx_core::command::spawn_command;
/// use std::path::Path;
///
/// // This works correctly even with paths like "C:\Program Files\nodejs\npm.cmd"
/// let output = spawn_command(Path::new("/usr/bin/npm"), &["--version"]).await?;
/// assert!(output.status.success());
/// ```
pub async fn spawn_command(executable: &Path, args: &[&str]) -> CommandResult<Output> {
    let mut cmd = build_command(executable, args);
    Ok(cmd.output().await?)
}

/// Spawn a command and inherit stdio (for interactive commands)
///
/// This is similar to `spawn_command` but inherits stdin/stdout/stderr
/// for interactive use.
///
/// # Arguments
///
/// * `executable` - Path to the executable
/// * `args` - Command arguments
///
/// # Returns
///
/// The exit status of the command
pub async fn spawn_command_inherit(
    executable: &Path,
    args: &[&str],
) -> CommandResult<std::process::ExitStatus> {
    let mut cmd = build_command(executable, args);
    cmd.stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());
    Ok(cmd.status().await?)
}

/// Build a command with proper cross-platform handling
///
/// Returns a configured `tokio::process::Command` that can be further customized.
///
/// # Arguments
///
/// * `executable` - Path to the executable
/// * `args` - Command arguments
///
/// # Returns
///
/// A configured `Command` ready for execution
pub fn build_command(executable: &Path, args: &[&str]) -> Command {
    #[cfg(windows)]
    {
        build_command_windows(executable, args)
    }

    #[cfg(not(windows))]
    {
        build_command_unix(executable, args)
    }
}

/// Build command for Windows
///
/// Handles `.cmd`/`.bat` files by using `cmd.exe /c` with `raw_arg`
/// for proper quoting.
#[cfg(windows)]
fn build_command_windows(executable: &Path, args: &[&str]) -> Command {
    let ext = executable
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "cmd" || ext == "bat" {
        let mut cmd = Command::new("cmd.exe");
        cmd.arg("/c");

        // Build the complete command string with proper quoting
        let cmd_string = build_cmd_string(executable, args);
        cmd.raw_arg(cmd_string);

        cmd
    } else {
        let mut cmd = Command::new(executable);
        cmd.args(args);
        cmd
    }
}

/// Build command for Unix systems
#[cfg(not(windows))]
fn build_command_unix(executable: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(executable);
    cmd.args(args);
    cmd
}

/// Build a command string for `cmd.exe /c` with proper quoting
///
/// Format: `"path with spaces" arg1 "arg with spaces"`
#[cfg(windows)]
fn build_cmd_string(executable: &Path, args: &[&str]) -> String {
    let mut parts = Vec::new();

    // Add executable (quote if contains spaces)
    let exe_str = executable.to_string_lossy();
    parts.push(quote_if_needed(&exe_str));

    // Add arguments
    for arg in args {
        parts.push(quote_if_needed(arg));
    }

    // Join with spaces
    parts.join(" ")
}

/// Quote a string if it contains spaces or special characters
///
/// This follows cmd.exe quoting rules (which differ from standard Windows quoting).
#[cfg(windows)]
fn quote_if_needed(s: &str) -> String {
    // Characters that require quoting in cmd.exe
    let needs_quoting = s.contains(' ')
        || s.contains('"')
        || s.contains('&')
        || s.contains('|')
        || s.contains('<')
        || s.contains('>')
        || s.contains('^');

    if needs_quoting {
        // Escape any existing quotes by doubling them
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

/// Check if an executable is a Windows batch file
pub fn is_batch_file(path: &Path) -> bool {
    if cfg!(windows) {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        ext == "cmd" || ext == "bat"
    } else {
        false
    }
}

/// Get the appropriate executable extension for the current platform
pub fn executable_extension() -> &'static str {
    if cfg!(windows) { ".exe" } else { "" }
}

/// Add the appropriate executable extension if needed
pub fn with_executable_extension(name: &str) -> String {
    if cfg!(windows)
        && !name.ends_with(".exe")
        && !name.ends_with(".cmd")
        && !name.ends_with(".bat")
    {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_batch_file() {
        if cfg!(windows) {
            assert!(is_batch_file(Path::new("test.cmd")));
            assert!(is_batch_file(Path::new("test.bat")));
            assert!(is_batch_file(Path::new("test.CMD")));
            assert!(!is_batch_file(Path::new("test.exe")));
            assert!(!is_batch_file(Path::new("test")));
        }
    }

    #[cfg(windows)]
    #[test]
    fn test_quote_if_needed() {
        assert_eq!(quote_if_needed("simple"), "simple");
        assert_eq!(quote_if_needed("with space"), "\"with space\"");
        assert_eq!(quote_if_needed("with&special"), "\"with&special\"");
        assert_eq!(quote_if_needed("with\"quote"), "\"with\"\"quote\"");
    }

    #[cfg(windows)]
    #[test]
    fn test_build_cmd_string() {
        let exe = Path::new("C:\\Program Files\\nodejs\\npm.cmd");
        let args = ["--version"];
        let cmd_string = build_cmd_string(exe, &args);
        assert!(cmd_string.starts_with("\"C:\\Program Files\\nodejs\\npm.cmd\""));
        assert!(cmd_string.contains("--version"));
    }
}
