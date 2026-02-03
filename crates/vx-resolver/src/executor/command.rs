//! Command building and execution
//!
//! This module handles:
//! - Building the command to execute
//! - Handling Windows .cmd/.bat files (using raw_arg for proper quoting)
//! - Running the command with timeout

use crate::Result;
use std::collections::HashMap;
use std::process::{ExitStatus, Stdio};
use tokio::process::Command;
use tracing::trace;

/// Build a command for execution
pub fn build_command(
    resolution: &crate::resolver::ResolutionResult,
    args: &[String],
    runtime_env: &HashMap<String, String>,
    inherit_vx_path: bool,
    vx_tools_path: Option<String>,
) -> Result<Command> {
    let executable = &resolution.executable;

    // On Windows, .cmd and .bat files need to be executed via cmd.exe
    // We use raw_arg to properly handle paths with spaces, because cmd.exe
    // doesn't follow standard CommandLineToArgvW escaping rules.
    //
    // The correct format for cmd /c with spaces is:
    //   cmd /c ""path with spaces" arg1 arg2"
    // The outer quotes are stripped by /c, leaving the inner quotes intact.
    #[cfg(windows)]
    let mut cmd = {
        let ext = executable
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext == "cmd" || ext == "bat" {
            let mut c = Command::new("cmd.exe");
            c.arg("/c");

            // Build the complete command string with proper quoting
            // Format: ""path" args..." where outer quotes are stripped by /c
            let cmd_string = build_cmd_string(executable, &resolution.command_prefix, args);
            c.raw_arg(cmd_string);

            // Return early since we've already added all arguments via raw_arg
            return finalize_command(c, runtime_env, inherit_vx_path, vx_tools_path, resolution);
        } else {
            Command::new(executable)
        }
    };

    #[cfg(not(windows))]
    let mut cmd = Command::new(executable);

    // Add command prefix if any (e.g., "msbuild" for dotnet)
    for prefix in &resolution.command_prefix {
        cmd.arg(prefix);
    }

    // Add user arguments
    cmd.args(args);

    finalize_command(cmd, runtime_env, inherit_vx_path, vx_tools_path, resolution)
}

/// Build a command string for cmd.exe /c with proper quoting
///
/// Format: ""path with spaces" arg1 "arg with spaces" arg2"
/// The outer quotes are needed because /c strips the first and last quote.
#[cfg(windows)]
fn build_cmd_string(
    executable: &std::path::Path,
    command_prefix: &[String],
    args: &[String],
) -> String {
    let mut parts = Vec::new();

    // Add executable (quote if contains spaces)
    let exe_str = executable.to_string_lossy();
    parts.push(quote_if_needed(&exe_str));

    // Add command prefix
    for prefix in command_prefix {
        parts.push(quote_if_needed(prefix));
    }

    // Add arguments
    for arg in args {
        parts.push(quote_if_needed(arg));
    }

    // Join with spaces
    parts.join(" ")
}

/// Quote a string if it contains spaces or special characters
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

/// Finalize command setup with environment variables and stdio
fn finalize_command(
    mut cmd: Command,
    runtime_env: &HashMap<String, String>,
    inherit_vx_path: bool,
    vx_tools_path: Option<String>,
    resolution: &crate::resolver::ResolutionResult,
) -> Result<Command> {
    // Build the final environment
    let mut final_env = runtime_env.clone();

    // If inherit_vx_path is enabled, prepend all vx-managed tool bin directories to PATH
    if inherit_vx_path {
        if let Some(vx_path) = vx_tools_path {
            let current_path = final_env
                .get("PATH")
                .cloned()
                .or_else(|| std::env::var("PATH").ok())
                .unwrap_or_default();

            let new_path = if current_path.is_empty() {
                vx_path
            } else {
                vx_paths::prepend_to_path(&current_path, &[vx_path])
            };

            final_env.insert("PATH".to_string(), new_path);
            trace!("PATH includes vx-managed tools for {}", resolution.runtime);
        }
    }

    // CRITICAL: Ensure essential system paths are always present in PATH
    #[cfg(unix)]
    {
        let current_path = final_env
            .get("PATH")
            .cloned()
            .or_else(|| std::env::var("PATH").ok())
            .unwrap_or_default();

        let mut path_parts: Vec<String> = vx_paths::split_path(&current_path)
            .map(String::from)
            .collect();

        let essential_paths = ["/bin", "/usr/bin", "/usr/local/bin"];
        let mut added_any = false;

        for essential in &essential_paths {
            let essential_str = essential.to_string();
            if !path_parts.iter().any(|p| p == &essential_str)
                && std::path::Path::new(essential).exists()
            {
                path_parts.push(essential_str);
                added_any = true;
            }
        }

        if added_any {
            let new_path = std::env::join_paths(&path_parts)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(current_path);
            final_env.insert("PATH".to_string(), new_path);
            trace!("Added essential system paths for child processes");
        }
    }

    // Inject environment variables
    if !final_env.is_empty() {
        trace!(
            "injecting {} env vars for {}",
            final_env.len(),
            resolution.runtime
        );
        for (key, value) in &final_env {
            cmd.env(key, value);
        }
    }

    // Inherit stdio
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    Ok(cmd)
}

/// Run the command and return its status
pub async fn run_command(
    cmd: &mut Command,
    timeout: Option<std::time::Duration>,
) -> Result<ExitStatus> {
    let status = if let Some(timeout) = timeout {
        tokio::time::timeout(timeout, cmd.status())
            .await
            .map_err(|_| anyhow::anyhow!("Command execution timed out"))??
    } else {
        cmd.status().await?
    };

    Ok(status)
}
