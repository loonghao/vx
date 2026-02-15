//! Command building and execution
//!
//! This module handles:
//! - Building the command to execute (with PATH debugging)
//! - Handling Windows .cmd/.bat files (using raw_arg for proper quoting)
//! - Running the command with timeout

use crate::Result;
use std::collections::HashMap;
use std::process::{ExitStatus, Stdio};
use tokio::process::Command;
use tracing::trace;

use super::pipeline::error::ExecuteError;

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
    if inherit_vx_path && let Some(vx_path) = vx_tools_path {
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

    // CRITICAL: Ensure essential system paths are always present in PATH
    // This is a safety net — even if upstream PATH construction has bugs,
    // child processes must always be able to find fundamental system executables
    // like cmd.exe (Windows) or sh/bash (Unix).
    {
        let current_path = final_env
            .get("PATH")
            .cloned()
            .or_else(|| std::env::var("PATH").ok())
            .unwrap_or_default();

        let mut path_parts: Vec<String> = vx_paths::split_path(&current_path)
            .map(String::from)
            .collect();

        let essential_paths = essential_system_paths();
        let mut added_any = false;

        for essential in &essential_paths {
            let essential_lower = essential.to_lowercase();
            if !path_parts
                .iter()
                .any(|p| p.to_lowercase() == essential_lower)
                && std::path::Path::new(essential).exists()
            {
                path_parts.push(essential.clone());
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

    // CRITICAL: Ensure essential Windows system environment variables are present.
    // When runtime_env contains explicit env vars, cmd.env() overwrites only those keys
    // while inheriting the rest from the parent process. However, some tools (e.g.,
    // node-gyp, python subprocess) rely on variables like SYSTEMROOT, COMSPEC, PATHEXT
    // to locate system executables (cmd.exe, powershell.exe). If any upstream code
    // accidentally clears or filters these, child processes break with errors like
    // "'cmd' is not recognized as an internal or external command".
    #[cfg(windows)]
    {
        for var_name in WINDOWS_ESSENTIAL_ENV_VARS {
            if !final_env.keys().any(|k| k.eq_ignore_ascii_case(var_name))
                && let Ok(value) = std::env::var(var_name)
            {
                final_env.insert(var_name.to_string(), value);
            }
        }
    }

    // Ensure vx executable's own directory is in PATH
    // This allows sub-processes (e.g., just recipes calling `vx npm ci`) to find vx
    if let Ok(current_exe) = std::env::current_exe()
        && let Some(exe_dir) = current_exe.parent()
    {
        let exe_dir_str = exe_dir.to_string_lossy().to_string();
        let current_path = final_env.get("PATH").cloned().unwrap_or_default();
        if !current_path
            .to_lowercase()
            .contains(&exe_dir_str.to_lowercase())
        {
            let sep = if cfg!(windows) { ";" } else { ":" };
            let new_path = if current_path.is_empty() {
                exe_dir_str
            } else {
                format!("{}{}{}", current_path, sep, exe_dir_str)
            };
            final_env.insert("PATH".to_string(), new_path);
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

/// Essential Windows environment variables that must always be present
/// for child processes to function correctly.
///
/// Without these, fundamental operations like `cmd /c "..."` or PowerShell
/// script execution will fail because the system cannot locate executables.
#[cfg(windows)]
const WINDOWS_ESSENTIAL_ENV_VARS: &[&str] = &[
    "SYSTEMROOT",             // C:\Windows — needed for cmd.exe, system DLLs
    "SYSTEMDRIVE",            // C: — base drive letter
    "WINDIR",                 // C:\Windows — legacy alias for SYSTEMROOT
    "COMSPEC",                // C:\Windows\System32\cmd.exe — default command processor
    "PATHEXT",                // .COM;.EXE;.BAT;.CMD;... — executable extensions
    "OS",                     // Windows_NT — OS identification
    "PROCESSOR_ARCHITECTURE", // AMD64/ARM64 — needed by build tools
    "NUMBER_OF_PROCESSORS",   // CPU count — used by parallel builds
];

/// Get essential system paths that must always be present in PATH.
///
/// These paths contain fundamental system executables (cmd.exe, powershell, sh, etc.)
/// that child processes expect to find.
fn essential_system_paths() -> Vec<String> {
    let mut paths = Vec::new();

    #[cfg(windows)]
    {
        // Try SYSTEMROOT env var first, fall back to hardcoded paths
        let system_root = std::env::var("SYSTEMROOT").unwrap_or_else(|_| r"C:\Windows".to_string());

        // System32 — contains cmd.exe, powershell.exe, and most system utilities
        let system32 = format!(r"{}\System32", system_root);
        paths.push(system32.clone());

        // Wbem — Windows Management Instrumentation tools
        paths.push(format!(r"{}\Wbem", system32));

        // Windows PowerShell 5.x
        paths.push(format!(r"{}\WindowsPowerShell\v1.0", system32));

        // SYSTEMROOT itself (contains some executables)
        paths.push(system_root);

        // PowerShell 7+ (if installed)
        if let Ok(pf) = std::env::var("ProgramFiles") {
            let ps7 = format!(r"{}\PowerShell\7", pf);
            if std::path::Path::new(&ps7).exists() {
                paths.push(ps7);
            }
        }
    }

    #[cfg(unix)]
    {
        paths.extend([
            "/bin".to_string(),
            "/usr/bin".to_string(),
            "/usr/local/bin".to_string(),
        ]);
    }

    paths
}

/// Run the command and return its status
pub async fn run_command(
    cmd: &mut Command,
    timeout: Option<std::time::Duration>,
) -> Result<ExitStatus> {
    let status = if let Some(timeout) = timeout {
        tokio::time::timeout(timeout, cmd.status())
            .await
            .map_err(|_| ExecuteError::Timeout {
                seconds: timeout.as_secs(),
            })??
    } else {
        cmd.status().await?
    };

    Ok(status)
}
