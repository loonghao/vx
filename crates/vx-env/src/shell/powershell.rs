//! PowerShell script generator

use std::collections::HashMap;

/// Escape a value for use in PowerShell single-quoted string
///
/// In PowerShell single-quoted strings:
/// - Single quotes are escaped by doubling them ('')
/// - All other characters are treated literally (no variable expansion, no escape sequences)
fn escape_single_quoted(value: &str) -> String {
    // In single-quoted strings, only single quotes need escaping (by doubling)
    value.replace('\'', "''")
}

/// Escape a command for use in PowerShell double-quoted string
///
/// In PowerShell double-quoted strings:
/// - Backtick (`) is the escape character
/// - Dollar sign ($) needs escaping to prevent variable expansion
/// - Double quote (") needs escaping with backtick
fn escape_double_quoted(value: &str) -> String {
    value
        .replace('`', "``") // Escape backticks first
        .replace('$', "`$") // Escape dollar signs
        .replace('"', "`\"") // Escape double quotes
}

/// Generate a PowerShell script that sets environment variables and executes a command
///
/// # Features
///
/// - Uses `$ErrorActionPreference = 'Stop'` for strict error handling
/// - Properly escapes single quotes (doubles them) in environment variable values
/// - Uses `$env:VAR = 'value'` syntax for environment variables (single-quoted for literal values)
/// - Executes command via `cmd /c` for shell command compatibility
/// - Properly escapes special characters in commands
pub fn generate_script(cmd: &str, env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    // Set error action preference for better error handling
    script.push_str("$ErrorActionPreference = 'Stop'\r\n");

    // Set environment variables using PowerShell syntax
    for (key, value) in env_vars {
        // Escape single quotes by doubling them for single-quoted string
        let escaped_value = escape_single_quoted(value);
        script.push_str(&format!("$env:{} = '{}'\r\n", key, escaped_value));
    }

    // Execute the command using cmd /c for shell commands
    // Escape the command for use in double-quoted string
    let escaped_cmd = escape_double_quoted(cmd);
    script.push_str(&format!("cmd /c \"{}\"\r\n", escaped_cmd));
    script.push_str("exit $LASTEXITCODE\r\n");

    script
}

/// Generate a PowerShell activation script for interactive shell use
///
/// Uses single-quoted strings for literal values, properly escaping single quotes.
pub fn generate_activation_script(env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    script.push_str("# vx environment activation script for PowerShell\r\n");
    script.push_str("# Dot-source this file: . (vx env activate --shell powershell)\r\n\r\n");

    for (key, value) in env_vars {
        // Escape single quotes for single-quoted string
        let escaped_value = escape_single_quoted(value);
        script.push_str(&format!("$env:{} = '{}'\r\n", key, escaped_value));
    }

    script
}

/// Generate a PowerShell script that executes a command directly (without cmd /c)
///
/// Use this for PowerShell-native commands or when cmd.exe compatibility is not needed.
/// Properly escapes environment variable values for single-quoted strings.
pub fn generate_native_script(cmd: &str, env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    script.push_str("$ErrorActionPreference = 'Stop'\r\n");

    for (key, value) in env_vars {
        // Escape single quotes for single-quoted string
        let escaped_value = escape_single_quoted(value);
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
    fn test_escape_single_quoted() {
        // Single quotes should be doubled
        assert_eq!(escape_single_quoted("test"), "test");
        assert_eq!(escape_single_quoted("it's"), "it''s");
        assert_eq!(escape_single_quoted("don't"), "don''t");
        assert_eq!(escape_single_quoted("'quoted'"), "''quoted''");

        // Double quotes should remain unchanged in single-quoted context
        assert_eq!(escape_single_quoted("say \"hello\""), "say \"hello\"");

        // Dollar signs should remain unchanged in single-quoted context
        assert_eq!(escape_single_quoted("$HOME"), "$HOME");

        // Backticks should remain unchanged in single-quoted context
        assert_eq!(escape_single_quoted("`test`"), "`test`");
    }

    #[test]
    fn test_escape_double_quoted() {
        // Backticks should be doubled first
        assert_eq!(escape_double_quoted("`"), "``");
        assert_eq!(escape_double_quoted("a`b"), "a``b");

        // Dollar signs should be escaped with backtick
        assert_eq!(escape_double_quoted("$HOME"), "`$HOME");
        assert_eq!(escape_double_quoted("value $var"), "value `$var");

        // Double quotes should be escaped with backtick
        assert_eq!(escape_double_quoted("\"hello\""), "`\"hello`\"");
        assert_eq!(escape_double_quoted("say \"hi\""), "say `\"hi`\"");

        // Complex combinations
        assert_eq!(
            escape_double_quoted("echo \"$HOME\""),
            "echo `\"`$HOME`\""
        );
    }

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
    fn test_escape_single_quotes_in_env() {
        let mut env = HashMap::new();
        env.insert("MSG".to_string(), "It's a test".to_string());

        let script = generate_script("echo", &env);
        assert!(script.contains("$env:MSG = 'It''s a test'"));
    }

    #[test]
    fn test_escape_special_chars_in_command() {
        let env = HashMap::new();

        // Test command with double quotes
        let script = generate_script(r#"echo "hello world""#, &env);
        assert!(script.contains("cmd /c \"echo `\"hello world`\"\""));

        // Test command with dollar sign (should be escaped in double-quoted context)
        let script = generate_script("echo $HOME", &env);
        assert!(script.contains("cmd /c \"echo `$HOME\""));

        // Test command with backtick
        let script = generate_script("echo `test`", &env);
        assert!(script.contains("cmd /c \"echo ``test``\""));
    }

    #[test]
    fn test_env_var_with_double_quotes() {
        let mut env = HashMap::new();
        env.insert("MSG".to_string(), r#"He said "hello""#.to_string());

        let script = generate_script("echo", &env);
        // In single-quoted string, double quotes are literal
        assert!(script.contains(r#"$env:MSG = 'He said "hello"'"#));
    }

    #[test]
    fn test_env_var_with_dollar_sign() {
        let mut env = HashMap::new();
        env.insert("PATTERN".to_string(), "$HOME/path".to_string());

        let script = generate_script("echo", &env);
        // In single-quoted string, dollar signs are literal (no variable expansion)
        assert!(script.contains("$env:PATTERN = '$HOME/path'"));
    }

    #[test]
    fn test_env_var_with_backtick() {
        let mut env = HashMap::new();
        env.insert("CODE".to_string(), "`test`".to_string());

        let script = generate_script("echo", &env);
        // In single-quoted string, backticks are literal
        assert!(script.contains("$env:CODE = '`test`'"));
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
    fn test_activation_script_with_special_chars() {
        let mut env = HashMap::new();
        env.insert("MSG".to_string(), "It's a \"test\" with $vars and `code`".to_string());

        let script = generate_activation_script(&env);
        // Only single quotes should be escaped in activation script values
        assert!(script.contains("$env:MSG = 'It''s a \"test\" with $vars and `code`'"));
    }

    #[test]
    fn test_native_script() {
        let env = HashMap::new();
        let script = generate_native_script("Get-Process", &env);

        assert!(!script.contains("cmd /c"));
        assert!(script.contains("Get-Process"));
    }

    #[test]
    fn test_crlf_line_endings() {
        let env = HashMap::new();
        let script = generate_script("echo test", &env);

        // Windows scripts should use CRLF line endings
        assert!(script.contains("\r\n"));
    }
}
