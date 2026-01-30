//! PowerShell script generator
//!
//! Generates PowerShell scripts for environment activation.
//! Follows the same design patterns as Python's venv and conda.

use super::ActivationConfig;
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

/// Generate a complete activation script with full virtual environment support
///
/// This follows the same design patterns as Python's venv and conda:
/// - Saves original PATH and prompt function
/// - Provides a `Vx-Deactivate` function to restore the environment
/// - Updates the prompt to show the environment name
/// - Prevents double activation
///
/// # Example
///
/// ```rust
/// use vx_env::shell::powershell::generate_full_activation_script;
/// use vx_env::shell::ActivationConfig;
///
/// let config = ActivationConfig::new("my-project")
///     .with_path("C:\\Users\\user\\.vx\\tools\\node\\bin")
///     .with_env("NODE_ENV", "development");
///
/// let script = generate_full_activation_script(&config);
/// // Script includes Vx-Deactivate function, prompt modification, etc.
/// ```
pub fn generate_full_activation_script(config: &ActivationConfig) -> String {
    let mut script = String::new();

    // Header
    script.push_str("# vx environment activation script for PowerShell\r\n");
    script.push_str("# Dot-source this file: . (vx env activate --shell powershell)\r\n");
    script.push_str("# Or: Invoke-Expression (vx dev --export --format powershell)\r\n\r\n");

    // Define deactivate function
    script.push_str(
        r#"# Deactivate function to restore previous environment
function global:Vx-Deactivate {
    [CmdletBinding()]
    param([switch]$NonDestructive)

    # Restore old PATH
    if (Test-Path variable:global:_OLD_VX_PATH) {
        $env:PATH = $global:_OLD_VX_PATH
        Remove-Variable -Name _OLD_VX_PATH -Scope Global
    }

    # Restore old prompt function
    if (Test-Path function:global:_old_vx_prompt) {
        $function:prompt = $function:_old_vx_prompt
        Remove-Item function:\_old_vx_prompt -ErrorAction SilentlyContinue
    }

    # Unset VX environment variables
    Remove-Item env:VX_ACTIVE -ErrorAction SilentlyContinue
    Remove-Item env:VX_PROJECT_NAME -ErrorAction SilentlyContinue
    Remove-Item env:VX_PROJECT_ROOT -ErrorAction SilentlyContinue

"#,
    );

    // Unset custom environment variables in deactivate
    for key in config.env_vars.keys() {
        if key != "PATH" && !key.starts_with("VX_") {
            script.push_str(&format!(
                "    Remove-Item env:{} -ErrorAction SilentlyContinue\r\n",
                key
            ));
        }
    }

    // Remove aliases in deactivate
    for alias_name in config.aliases.keys() {
        script.push_str(&format!(
            "    Remove-Item alias:{} -ErrorAction SilentlyContinue\r\n",
            alias_name
        ));
    }

    script.push_str(
        r#"
    # Self-destruct (unless NonDestructive)
    if (-not $NonDestructive) {
        Remove-Item function:Vx-Deactivate -ErrorAction SilentlyContinue
    }
}

"#,
    );

    // Check for double activation
    script.push_str(
        r#"# Prevent double activation
if ($env:VX_ACTIVE) {
    Write-Warning "vx environment is already active. Run 'Vx-Deactivate' first."
    return
}

"#,
    );

    // Save current environment
    script.push_str(
        r#"# Save current environment
$global:_OLD_VX_PATH = $env:PATH
if (Test-Path function:prompt) {
    $function:global:_old_vx_prompt = $function:prompt
}

"#,
    );

    // Update PATH
    if !config.path_entries.is_empty() {
        let paths = config.path_entries.join(";");
        let escaped_paths = escape_single_quoted(&paths);
        script.push_str(&format!(
            "# Add tool paths\r\n$env:PATH = '{}' + ';' + $env:PATH\r\n\r\n",
            escaped_paths
        ));
    }

    // Set VX environment marker
    script.push_str("# VX environment marker\r\n");
    script.push_str("$env:VX_ACTIVE = '1'\r\n");

    // Set project name if available
    if let Some(name) = &config.name {
        let escaped = escape_single_quoted(name);
        script.push_str(&format!("$env:VX_PROJECT_NAME = '{}'\r\n", escaped));
    }

    // Set custom environment variables
    if !config.env_vars.is_empty() {
        script.push_str("\r\n# Custom environment variables\r\n");
        for (key, value) in &config.env_vars {
            if key == "PATH" {
                continue; // PATH is handled separately
            }
            let escaped = escape_single_quoted(value);
            script.push_str(&format!("$env:{} = '{}'\r\n", key, escaped));
        }
    }

    // Define aliases
    if !config.aliases.is_empty() {
        script.push_str("\r\n# Shell aliases\r\n");
        for (name, command) in &config.aliases {
            let escaped = escape_single_quoted(command);
            script.push_str(&format!(
                "Set-Alias -Name {} -Value '{}' -Scope Global\r\n",
                name, escaped
            ));
        }
    }

    // Update prompt
    let prompt_prefix = config.prompt_prefix();
    let escaped_prefix = escape_single_quoted(&prompt_prefix);
    script.push_str(&format!(
        r#"
# Update prompt
function global:prompt {{
    $previous_prompt = if (Test-Path function:global:_old_vx_prompt) {{
        & $function:_old_vx_prompt
    }} else {{
        "PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
    }}
    "{} $previous_prompt"
}}

# Type 'Vx-Deactivate' to exit the vx environment
"#,
        escaped_prefix
    ));

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
        assert_eq!(escape_double_quoted("echo \"$HOME\""), "echo `\"`$HOME`\"");
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
        env.insert(
            "MSG".to_string(),
            "It's a \"test\" with $vars and `code`".to_string(),
        );

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

    #[test]
    fn test_full_activation_script_basic() {
        let config = ActivationConfig::new("my-project");
        let script = generate_full_activation_script(&config);

        // Check header
        assert!(script.contains("# vx environment activation script for PowerShell"));

        // Check deactivate function
        assert!(script.contains("function global:Vx-Deactivate"));
        assert!(script.contains("$env:PATH = $global:_OLD_VX_PATH"));
        assert!(script.contains("Remove-Item env:VX_ACTIVE"));

        // Check double activation prevention
        assert!(script.contains("if ($env:VX_ACTIVE)"));

        // Check environment saving
        assert!(script.contains("$global:_OLD_VX_PATH = $env:PATH"));

        // Check VX marker
        assert!(script.contains("$env:VX_ACTIVE = '1'"));
        assert!(script.contains("$env:VX_PROJECT_NAME = 'my-project'"));

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
        assert!(script.contains("$env:PATH = 'C:\\tools\\bin;C:\\node\\bin' + ';' + $env:PATH"));
    }

    #[test]
    fn test_full_activation_script_with_env_vars() {
        let config = ActivationConfig::new("test")
            .with_env("NODE_ENV", "development")
            .with_env("DEBUG", "true");

        let script = generate_full_activation_script(&config);
        assert!(script.contains("$env:NODE_ENV = 'development'"));
        assert!(script.contains("$env:DEBUG = 'true'"));

        // Check env vars are removed in deactivate
        assert!(script.contains("Remove-Item env:NODE_ENV"));
        assert!(script.contains("Remove-Item env:DEBUG"));
    }

    #[test]
    fn test_full_activation_script_with_aliases() {
        let config = ActivationConfig::new("test")
            .with_alias("ll", "Get-ChildItem")
            .with_alias("gs", "git status");

        let script = generate_full_activation_script(&config);
        assert!(script.contains("Set-Alias -Name ll -Value 'Get-ChildItem' -Scope Global"));
        assert!(script.contains("Set-Alias -Name gs -Value 'git status' -Scope Global"));

        // Check aliases are removed in deactivate
        assert!(script.contains("Remove-Item alias:ll"));
        assert!(script.contains("Remove-Item alias:gs"));
    }

    #[test]
    fn test_full_activation_script_custom_prompt() {
        let config = ActivationConfig::new("myenv").with_prompt_format("[{name}]");

        let script = generate_full_activation_script(&config);
        assert!(script.contains("[myenv]"));
    }

    #[test]
    fn test_full_activation_script_special_chars() {
        let config = ActivationConfig::new("test").with_env("MSG", "It's a test");

        let script = generate_full_activation_script(&config);
        // Single quotes should be escaped
        assert!(script.contains("$env:MSG = 'It''s a test'"));
    }
}
