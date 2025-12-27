//! PowerShell script generator

use std::collections::HashMap;

/// Generate a PowerShell script that sets environment variables and executes a command
///
/// # Features
///
/// - Uses `$ErrorActionPreference = 'Stop'` for strict error handling
/// - Properly escapes single quotes (doubles them)
/// - Uses `$env:VAR` syntax for environment variables
/// - Executes command via `cmd /c` for shell command compatibility
pub fn generate_script(cmd: &str, env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    // Set error action preference for better error handling
    script.push_str("$ErrorActionPreference = 'Stop'\r\n");

    // Set environment variables using PowerShell syntax
    for (key, value) in env_vars {
        // Escape single quotes by doubling them
        let escaped_value = value.replace('\'', "''");
        script.push_str(&format!("$env:{} = '{}'\r\n", key, escaped_value));
    }

    // Execute the command using cmd /c for shell commands
    script.push_str(&format!("cmd /c \"{}\"\r\n", cmd.replace('"', "\\\"")));
    script.push_str("exit $LASTEXITCODE\r\n");

    script
}

/// Generate a PowerShell activation script for interactive shell use
pub fn generate_activation_script(env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    script.push_str("# vx environment activation script for PowerShell\r\n");
    script.push_str("# Dot-source this file: . (vx env activate --shell powershell)\r\n\r\n");

    for (key, value) in env_vars {
        let escaped_value = value.replace('\'', "''");
        script.push_str(&format!("$env:{} = '{}'\r\n", key, escaped_value));
    }

    script
}

/// Generate a PowerShell script that executes a command directly (without cmd /c)
///
/// Use this for PowerShell-native commands or when cmd.exe compatibility is not needed.
pub fn generate_native_script(cmd: &str, env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    script.push_str("$ErrorActionPreference = 'Stop'\r\n");

    for (key, value) in env_vars {
        let escaped_value = value.replace('\'', "''");
        script.push_str(&format!("$env:{} = '{}'\r\n", key, escaped_value));
    }

    // Execute directly without cmd /c
    script.push_str(&format!("\r\n{}\r\n", cmd));

    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_script_basic() {
        let env = HashMap::new();
        let script = generate_script("echo hello", &env);

        assert!(script.contains("$ErrorActionPreference = 'Stop'"));
        assert!(script.contains("cmd /c"));
        assert!(script.contains("echo hello"));
        assert!(script.contains("exit $LASTEXITCODE"));
    }

    #[test]
    fn test_generate_script_with_env() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());

        let script = generate_script("echo %FOO%", &env);
        assert!(script.contains("$env:FOO = 'bar'"));
    }

    #[test]
    fn test_escape_single_quotes() {
        let mut env = HashMap::new();
        env.insert("MSG".to_string(), "It's a test".to_string());

        let script = generate_script("echo", &env);
        assert!(script.contains("It''s a test"));
    }

    #[test]
    fn test_activation_script() {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "C:\\tools\\bin".to_string());

        let script = generate_activation_script(&env);
        assert!(script.contains("$env:PATH = 'C:\\tools\\bin'"));
        assert!(script.contains("# vx environment activation script"));
    }

    #[test]
    fn test_native_script() {
        let env = HashMap::new();
        let script = generate_native_script("Get-Process", &env);

        assert!(!script.contains("cmd /c"));
        assert!(script.contains("Get-Process"));
    }
}
