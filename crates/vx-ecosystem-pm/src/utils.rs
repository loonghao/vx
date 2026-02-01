//! Utility functions for ecosystem installers

use crate::types::InstallEnv;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

/// Run a command with the specified environment
///
/// # Arguments
/// * `command` - The command to run
/// * `args` - Command arguments
/// * `env` - Environment configuration
/// * `verbose` - Whether to show output in real-time
///
/// # Returns
/// The command output
pub fn run_command(command: &str, args: &[&str], env: &InstallEnv, verbose: bool) -> Result<Output> {
    let mut cmd = Command::new(command);
    cmd.args(args);

    // Set environment variables
    for (key, value) in &env.vars {
        cmd.env(key, value);
    }

    // Prepend to PATH if needed
    if !env.path_prepend.is_empty() {
        let current_path = std::env::var("PATH").unwrap_or_default();
        let path_sep = if cfg!(windows) { ";" } else { ":" };
        let new_path = env
            .path_prepend
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(path_sep);
        let full_path = format!("{}{}{}", new_path, path_sep, current_path);
        cmd.env("PATH", full_path);
    }

    if verbose {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    }

    let output = cmd.output().with_context(|| {
        format!(
            "Failed to execute command: {} {}",
            command,
            args.join(" ")
        )
    })?;

    Ok(output)
}

/// Detect executables in a directory
///
/// # Arguments
/// * `bin_dir` - The directory to scan for executables
///
/// # Returns
/// List of executable names (without extensions on Windows)
pub fn detect_executables_in_dir(bin_dir: &PathBuf) -> Result<Vec<String>> {
    let mut executables = Vec::new();

    if !bin_dir.exists() {
        return Ok(executables);
    }

    for entry in std::fs::read_dir(bin_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip common non-executable files
                if name.ends_with(".ps1") || name.ends_with(".md") || name.ends_with(".txt") {
                    continue;
                }

                // On Windows, check for executable extensions
                #[cfg(windows)]
                {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if matches!(ext.to_lowercase().as_str(), "exe" | "cmd" | "bat") {
                            // Remove extension for the executable name
                            let exe_name = name.strip_suffix(&format!(".{}", ext)).unwrap_or(name);
                            if !executables.contains(&exe_name.to_string()) {
                                executables.push(exe_name.to_string());
                            }
                        }
                    }
                }

                // On Unix, check for executable permission
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = std::fs::metadata(&path) {
                        if meta.permissions().mode() & 0o111 != 0 {
                            executables.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    executables.sort();
    Ok(executables)
}

