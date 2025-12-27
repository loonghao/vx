//! Script executor for platform-specific wrapper scripts
//!
//! This module provides functions to generate and execute platform-specific
//! wrapper scripts that set up environment variables before running commands.
//! Inspired by rez's shell execution model.

use crate::error::EnvError;
use crate::shell;
use std::collections::HashMap;

/// Execute a command by generating a platform-specific wrapper script
///
/// This approach (inspired by rez's shell execution model) generates a temporary
/// script that sets up the environment and then executes the command. This ensures
/// that environment variables like PATH are properly available to the command and
/// any subprocesses it spawns.
///
/// # Platform-specific shells
///
/// - Windows: PowerShell (pwsh/powershell) - modern default, better error handling
/// - Linux/macOS: bash - standard default with pipefail support
///
/// # Example
///
/// ```rust,no_run
/// use vx_env::execute_with_env;
/// use std::collections::HashMap;
///
/// let mut env = HashMap::new();
/// env.insert("MY_VAR".to_string(), "my_value".to_string());
///
/// let status = execute_with_env("echo $MY_VAR", &env).unwrap();
/// assert!(status.success());
/// ```
pub fn execute_with_env(
    cmd: &str,
    env_vars: &HashMap<String, String>,
) -> Result<std::process::ExitStatus, EnvError> {
    use std::fs;
    use std::io::Write;
    use std::process::Command;

    // Create a temporary directory for the script
    let temp_dir = std::env::temp_dir();
    let script_id = std::process::id();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    #[cfg(windows)]
    let script_path = temp_dir.join(format!("vx_run_{}_{}.ps1", script_id, timestamp));

    #[cfg(not(windows))]
    let script_path = temp_dir.join(format!("vx_run_{}_{}.sh", script_id, timestamp));

    // Generate the script content
    let script_content = generate_wrapper_script(cmd, env_vars);

    // Write the script
    {
        let mut file = fs::File::create(&script_path)?;
        file.write_all(script_content.as_bytes())?;
    }

    // Make executable on Unix
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    // Execute the script using platform-appropriate shell
    #[cfg(windows)]
    let status = {
        // Try pwsh (PowerShell Core) first, fall back to powershell (Windows PowerShell)
        let script_path_str = script_path.to_string_lossy();
        let pwsh_result = Command::new("pwsh")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &script_path_str,
            ])
            .status();

        match pwsh_result {
            Ok(status) => Ok(status),
            Err(_) => {
                // Fall back to Windows PowerShell
                Command::new("powershell")
                    .args([
                        "-NoProfile",
                        "-NonInteractive",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                        &script_path_str,
                    ])
                    .status()
            }
        }
    };

    #[cfg(not(windows))]
    let status = {
        // Use bash with pipefail for better error handling
        // Fall back to sh if bash is not available
        let bash_result = Command::new("bash").arg(&script_path).status();

        match bash_result {
            Ok(status) => Ok(status),
            Err(_) => {
                // Fall back to sh
                Command::new("sh").arg(&script_path).status()
            }
        }
    };

    // Clean up the temporary script
    let _ = fs::remove_file(&script_path);

    status.map_err(|e| EnvError::Execution(e.to_string()))
}

/// Generate a platform-specific wrapper script that sets environment variables
/// and executes the command
///
/// # Platform-specific formats
///
/// - Windows: PowerShell script (.ps1) with $env:VAR syntax
/// - Linux/macOS: Bash script (.sh) with export VAR syntax
///
/// # Example
///
/// ```rust
/// use vx_env::generate_wrapper_script;
/// use std::collections::HashMap;
///
/// let mut env = HashMap::new();
/// env.insert("NODE_ENV".to_string(), "production".to_string());
///
/// let script = generate_wrapper_script("node app.js", &env);
/// // On Unix: contains "export NODE_ENV='production'"
/// // On Windows: contains "$env:NODE_ENV = 'production'"
/// ```
pub fn generate_wrapper_script(cmd: &str, env_vars: &HashMap<String, String>) -> String {
    #[cfg(windows)]
    {
        shell::powershell::generate_script(cmd, env_vars)
    }

    #[cfg(not(windows))]
    {
        shell::bash::generate_script(cmd, env_vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_script_basic() {
        let env_vars: HashMap<String, String> = HashMap::new();
        let script = generate_wrapper_script("echo hello", &env_vars);

        #[cfg(windows)]
        {
            assert!(script.contains("$ErrorActionPreference"));
            assert!(script.contains("echo hello"));
        }

        #[cfg(not(windows))]
        {
            assert!(script.contains("#!/usr/bin/env bash"));
            assert!(script.contains("echo hello"));
        }
    }

    #[test]
    fn test_generate_script_with_env() {
        let mut env_vars: HashMap<String, String> = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

        let script = generate_wrapper_script("echo $TEST_VAR", &env_vars);

        #[cfg(windows)]
        {
            assert!(script.contains("$env:TEST_VAR = 'test_value'"));
        }

        #[cfg(not(windows))]
        {
            assert!(script.contains("export TEST_VAR='test_value'"));
        }
    }

    #[test]
    fn test_escape_single_quotes() {
        let mut env_vars: HashMap<String, String> = HashMap::new();
        env_vars.insert("MSG".to_string(), "It's working".to_string());

        let script = generate_wrapper_script("echo", &env_vars);

        #[cfg(windows)]
        {
            // PowerShell doubles single quotes
            assert!(script.contains("It''s working"));
        }

        #[cfg(not(windows))]
        {
            // Bash uses '\'' to escape
            assert!(script.contains("It'\\''s working"));
        }
    }
}
