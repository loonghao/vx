//! Windows Command Prompt (cmd.exe) script generator
//!
//! Generates batch scripts for environment activation.
//! Follows the same design patterns as Python's venv and conda.
//!
//! Note: CMD has limitations compared to PowerShell/Bash:
//! - No function definitions (deactivate requires a separate batch file)
//! - Limited prompt customization
//! - No easy way to unset variables

use super::ActivationConfig;
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

/// Generate a batch activation script for interactive shell use (legacy API)
///
/// **Note**: For full virtual environment support, use `generate_full_activation_script` instead.
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

/// Generate a complete activation script with full virtual environment support
///
/// This follows the same design patterns as Python's venv and conda:
/// - Saves original PATH and PROMPT
/// - Provides instructions for deactivation
/// - Updates the prompt to show the environment name
/// - Prevents double activation
///
/// Note: CMD batch files have limitations:
/// - Cannot define functions, so deactivation is done via `set PATH=%_OLD_VX_PATH%`
/// - Or use the generated deactivate.bat file
///
/// # Example
///
/// ```rust
/// use vx_env::shell::cmd::generate_full_activation_script;
/// use vx_env::shell::ActivationConfig;
///
/// let config = ActivationConfig::new("my-project")
///     .with_path("C:\\Users\\user\\.vx\\tools\\node\\bin")
///     .with_env("NODE_ENV", "development");
///
/// let script = generate_full_activation_script(&config);
/// ```
pub fn generate_full_activation_script(config: &ActivationConfig) -> String {
    let mut script = String::new();

    // Header
    script.push_str("@echo off\r\n");
    script.push_str("REM vx environment activation script for cmd.exe\r\n");
    script.push_str("REM Run: vx env activate --shell cmd > activate.bat && activate.bat\r\n\r\n");

    // Check for double activation
    script.push_str("REM Prevent double activation\r\n");
    script.push_str("if defined VX_ACTIVE (\r\n");
    script.push_str("    echo Warning: vx environment is already active.\r\n");
    script.push_str("    echo To deactivate, run: set PATH=%%_OLD_VX_PATH%%\r\n");
    script.push_str("    goto :eof\r\n");
    script.push_str(")\r\n\r\n");

    // Save current environment
    script.push_str("REM Save current environment\r\n");
    script.push_str("set \"_OLD_VX_PATH=%PATH%\"\r\n");
    script.push_str("set \"_OLD_VX_PROMPT=%PROMPT%\"\r\n\r\n");

    // Update PATH
    if !config.path_entries.is_empty() {
        let paths = config.path_entries.join(";");
        let escaped_paths = escape_batch_value(&paths);
        script.push_str("REM Add tool paths\r\n");
        script.push_str(&format!("set \"PATH={};%PATH%\"\r\n\r\n", escaped_paths));
    }

    // Set VX environment marker
    script.push_str("REM VX environment marker\r\n");
    script.push_str("set \"VX_ACTIVE=1\"\r\n");

    // Set project name if available
    if let Some(name) = &config.name {
        let escaped = escape_batch_value(name);
        script.push_str(&format!("set \"VX_PROJECT_NAME={}\"\r\n", escaped));
    }

    // Set custom environment variables
    if !config.env_vars.is_empty() {
        script.push_str("\r\nREM Custom environment variables\r\n");
        for (key, value) in &config.env_vars {
            if key == "PATH" {
                continue; // PATH is handled separately
            }
            let escaped = escape_batch_value(value);
            script.push_str(&format!("set \"{}={}\"\r\n", key, escaped));
        }
    }

    // Update prompt
    let prompt_prefix = config.prompt_prefix();
    let escaped_prefix = escape_batch_value(&prompt_prefix);
    script.push_str(&format!(
        "\r\nREM Update prompt\r\nset \"PROMPT={} $P$G\"\r\n",
        escaped_prefix
    ));

    // Deactivation instructions
    script.push_str("\r\nREM To deactivate, run:\r\n");
    script.push_str("REM   set PATH=%_OLD_VX_PATH%\r\n");
    script.push_str("REM   set PROMPT=%_OLD_VX_PROMPT%\r\n");
    script.push_str("REM   set VX_ACTIVE=\r\n");

    script
}

/// Generate a deactivation script for cmd.exe
///
/// Since CMD doesn't support functions, this generates a separate batch file
/// that can be run to deactivate the environment.
pub fn generate_deactivation_script(config: &ActivationConfig) -> String {
    let mut script = String::new();

    script.push_str("@echo off\r\n");
    script.push_str("REM vx environment deactivation script for cmd.exe\r\n\r\n");

    // Check if environment is active
    script.push_str("if not defined VX_ACTIVE (\r\n");
    script.push_str("    echo No vx environment is currently active.\r\n");
    script.push_str("    goto :eof\r\n");
    script.push_str(")\r\n\r\n");

    // Restore PATH
    script.push_str("REM Restore original PATH\r\n");
    script.push_str("if defined _OLD_VX_PATH (\r\n");
    script.push_str("    set \"PATH=%_OLD_VX_PATH%\"\r\n");
    script.push_str("    set \"_OLD_VX_PATH=\"\r\n");
    script.push_str(")\r\n\r\n");

    // Restore PROMPT
    script.push_str("REM Restore original PROMPT\r\n");
    script.push_str("if defined _OLD_VX_PROMPT (\r\n");
    script.push_str("    set \"PROMPT=%_OLD_VX_PROMPT%\"\r\n");
    script.push_str("    set \"_OLD_VX_PROMPT=\"\r\n");
    script.push_str(")\r\n\r\n");

    // Unset VX environment variables
    script.push_str("REM Unset VX environment variables\r\n");
    script.push_str("set \"VX_ACTIVE=\"\r\n");
    script.push_str("set \"VX_PROJECT_NAME=\"\r\n");
    script.push_str("set \"VX_PROJECT_ROOT=\"\r\n");

    // Unset custom environment variables
    if !config.env_vars.is_empty() {
        script.push_str("\r\nREM Unset custom environment variables\r\n");
        for key in config.env_vars.keys() {
            if key == "PATH" || key.starts_with("VX_") {
                continue;
            }
            script.push_str(&format!("set \"{}=\"\r\n", key));
        }
    }

    script.push_str("\r\necho vx environment deactivated.\r\n");

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

    #[test]
    fn test_full_activation_script_basic() {
        let config = ActivationConfig::new("my-project");
        let script = generate_full_activation_script(&config);

        // Check header
        assert!(script.contains("REM vx environment activation script for cmd.exe"));

        // Check double activation prevention
        assert!(script.contains("if defined VX_ACTIVE"));

        // Check environment saving
        assert!(script.contains("set \"_OLD_VX_PATH=%PATH%\""));
        assert!(script.contains("set \"_OLD_VX_PROMPT=%PROMPT%\""));

        // Check VX marker
        assert!(script.contains("set \"VX_ACTIVE=1\""));
        assert!(script.contains("set \"VX_PROJECT_NAME=my-project\""));

        // Check prompt update
        assert!(script.contains("(my-project[vx])"));

        // Check CRLF line endings
        assert!(script.contains("\r\n"));
    }

    #[test]
    fn test_full_activation_script_with_paths() {
        let config = ActivationConfig::new("test")
            .with_path("C:\\tools\\bin")
            .with_path("C:\\node\\bin");

        let script = generate_full_activation_script(&config);
        assert!(script.contains("set \"PATH=C:\\tools\\bin;C:\\node\\bin;%PATH%\""));
    }

    #[test]
    fn test_full_activation_script_with_env_vars() {
        let config = ActivationConfig::new("test")
            .with_env("NODE_ENV", "development")
            .with_env("DEBUG", "true");

        let script = generate_full_activation_script(&config);
        assert!(script.contains("set \"NODE_ENV=development\""));
        assert!(script.contains("set \"DEBUG=true\""));
    }

    #[test]
    fn test_full_activation_script_special_chars() {
        let config = ActivationConfig::new("test").with_env("MSG", "100% complete");

        let script = generate_full_activation_script(&config);
        // Percent should be escaped
        assert!(script.contains("set \"MSG=100%% complete\""));
    }

    #[test]
    fn test_deactivation_script_basic() {
        let config = ActivationConfig::new("my-project");
        let script = generate_deactivation_script(&config);

        // Check header
        assert!(script.contains("REM vx environment deactivation script"));

        // Check if environment is active
        assert!(script.contains("if not defined VX_ACTIVE"));

        // Check PATH restoration
        assert!(script.contains("set \"PATH=%_OLD_VX_PATH%\""));

        // Check PROMPT restoration
        assert!(script.contains("set \"PROMPT=%_OLD_VX_PROMPT%\""));

        // Check VX variables are unset
        assert!(script.contains("set \"VX_ACTIVE=\""));
        assert!(script.contains("set \"VX_PROJECT_NAME=\""));
    }

    #[test]
    fn test_deactivation_script_with_env_vars() {
        let config = ActivationConfig::new("test")
            .with_env("NODE_ENV", "development")
            .with_env("DEBUG", "true");

        let script = generate_deactivation_script(&config);

        // Check custom env vars are unset
        assert!(script.contains("set \"NODE_ENV=\""));
        assert!(script.contains("set \"DEBUG=\""));
    }
}
