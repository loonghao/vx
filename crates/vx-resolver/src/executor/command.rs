//! Command building and execution
//!
//! This module handles:
//! - Building the command to execute
//! - Handling Windows .cmd/.bat files
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
    #[cfg(windows)]
    let mut cmd = {
        let ext = executable
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext == "cmd" || ext == "bat" {
            let mut c = Command::new("cmd.exe");
            // Use quoted path to handle spaces in path (e.g., "C:\Program Files\...")
            // cmd.exe /c requires the entire command to be quoted if it contains spaces
            let exe_str = executable.to_string_lossy();
            if exe_str.contains(' ') {
                c.arg("/c").arg(format!("\"{}\"", exe_str));
            } else {
                c.arg("/c").arg(executable);
            }
            c
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
