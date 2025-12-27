//! Windows Command Prompt (cmd.exe) script generator

use std::collections::HashMap;

/// Generate a batch script that sets environment variables and executes a command
///
/// # Features
///
/// - Uses `@echo off` to suppress command echoing
/// - Uses `set VAR=value` syntax for environment variables
/// - Properly escapes special characters
pub fn generate_script(cmd: &str, env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    // Suppress command echoing
    script.push_str("@echo off\r\n");

    // Set environment variables
    for (key, value) in env_vars {
        // Escape special characters for batch
        let escaped_value = escape_batch_value(value);
        script.push_str(&format!("set \"{}={}\"\r\n", key, escaped_value));
    }

    // Execute the command
    script.push_str(&format!("\r\n{}\r\n", cmd));

    script
}

/// Escape special characters for batch file values
fn escape_batch_value(value: &str) -> String {
    // Characters that need escaping in batch: & | < > ^ %
    // % needs to be doubled
    value
        .replace('%', "%%")
        .replace('^', "^^")
        .replace('&', "^&")
        .replace('|', "^|")
        .replace('<', "^<")
        .replace('>', "^>")
}

/// Generate a batch activation script for interactive shell use
pub fn generate_activation_script(env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    script.push_str("@echo off\r\n");
    script.push_str("REM vx environment activation script for cmd.exe\r\n");
    script.push_str(
        "REM Run this file: vx env activate --shell cmd > activate.bat && activate.bat\r\n\r\n",
    );

    for (key, value) in env_vars {
        let escaped_value = escape_batch_value(value);
        script.push_str(&format!("set \"{}={}\"\r\n", key, escaped_value));
    }

    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_script_basic() {
        let env = HashMap::new();
        let script = generate_script("echo hello", &env);

        assert!(script.contains("@echo off"));
        assert!(script.contains("echo hello"));
    }

    #[test]
    fn test_generate_script_with_env() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());

        let script = generate_script("echo %FOO%", &env);
        assert!(script.contains("set \"FOO=bar\""));
    }

    #[test]
    fn test_escape_special_chars() {
        let mut env = HashMap::new();
        env.insert("CMD".to_string(), "a & b | c".to_string());

        let script = generate_script("echo", &env);
        assert!(script.contains("a ^& b ^| c"));
    }

    #[test]
    fn test_escape_percent() {
        let mut env = HashMap::new();
        env.insert("MSG".to_string(), "100%".to_string());

        let script = generate_script("echo", &env);
        assert!(script.contains("100%%"));
    }

    #[test]
    fn test_activation_script() {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "C:\\tools\\bin".to_string());

        let script = generate_activation_script(&env);
        assert!(script.contains("set \"PATH=C:\\tools\\bin\""));
        assert!(script.contains("REM vx environment activation script"));
    }
}
