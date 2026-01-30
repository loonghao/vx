//! Bash script generator
//!
//! Generates bash/zsh compatible scripts for environment activation.
//! Follows the same design patterns as Python's venv and conda.

use super::ActivationConfig;
use std::collections::HashMap;

/// Escape a value for use in bash single-quoted string
///
/// In bash single-quoted strings, only single quotes need escaping.
/// The escape sequence is: end quote, escaped quote, start quote ('\'')
fn escape_single_quoted(value: &str) -> String {
    value.replace('\'', "'\\''")
}

/// Generate a bash script that sets environment variables and executes a command
///
/// # Features
///
/// - Uses `set -euo pipefail` for strict error handling
/// - Properly escapes single quotes in values
/// - Uses `export` for environment variables
pub fn generate_script(cmd: &str, env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    // Bash script with strict error handling
    script.push_str("#!/usr/bin/env bash\n");
    script.push_str("set -euo pipefail\n\n");

    // Set environment variables
    for (key, value) in env_vars {
        let escaped_value = escape_single_quoted(value);
        script.push_str(&format!("export {}='{}'\n", key, escaped_value));
    }

    // Execute the command
    script.push_str(&format!("\n{}\n", cmd));

    script
}

/// Generate an activation script for interactive shell use (legacy API)
///
/// Unlike `generate_script`, this doesn't execute a command but sets up
/// the environment for interactive use.
///
/// **Note**: For full virtual environment support, use `generate_full_activation_script` instead.
pub fn generate_activation_script(env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    script.push_str("# vx environment activation script\n");
    script.push_str("# Source this file: source <(vx env activate)\n\n");

    for (key, value) in env_vars {
        let escaped_value = escape_single_quoted(value);
        script.push_str(&format!("export {}='{}'\n", key, escaped_value));
    }

    script
}

/// Generate a complete activation script with full virtual environment support
///
/// This follows the same design patterns as Python's venv and conda:
/// - Saves original PATH and PS1
/// - Provides a `vx_deactivate` function to restore the environment
/// - Updates the prompt to show the environment name
/// - Prevents double activation
///
/// # Example
///
/// ```rust
/// use vx_env::shell::bash::generate_full_activation_script;
/// use vx_env::shell::ActivationConfig;
///
/// let config = ActivationConfig::new("my-project")
///     .with_path("/home/user/.vx/tools/node/bin")
///     .with_env("NODE_ENV", "development");
///
/// let script = generate_full_activation_script(&config);
/// // Script includes deactivate function, prompt modification, etc.
/// ```
pub fn generate_full_activation_script(config: &ActivationConfig) -> String {
    let mut script = String::new();

    // Header
    script.push_str("# vx environment activation script for Bash/Zsh\n");
    script.push_str("# Source this file: source <(vx env activate)\n");
    script.push_str("# Or: eval \"$(vx dev --export)\"\n\n");

    // Define deactivate function (similar to venv)
    script.push_str(
        r#"# Deactivate function to restore previous environment
vx_deactivate() {
    # Restore old PATH
    if [ -n "${_OLD_VX_PATH:-}" ]; then
        export PATH="$_OLD_VX_PATH"
        unset _OLD_VX_PATH
    fi

    # Restore old PS1 prompt
    if [ -n "${_OLD_VX_PS1:-}" ]; then
        export PS1="$_OLD_VX_PS1"
        unset _OLD_VX_PS1
    fi

    # Unset VX environment variables
    unset VX_ACTIVE
    unset VX_PROJECT_NAME
    unset VX_PROJECT_ROOT

"#,
    );

    // Unset custom environment variables in deactivate
    for key in config.env_vars.keys() {
        if key != "PATH" && !key.starts_with("VX_") {
            script.push_str(&format!("    unset {}\n", key));
        }
    }

    // Unset aliases in deactivate
    for alias_name in config.aliases.keys() {
        script.push_str(&format!("    unalias {} 2>/dev/null || true\n", alias_name));
    }

    script.push_str(
        r#"
    # Self-destruct
    unset -f vx_deactivate
}

"#,
    );

    // Check for double activation
    script.push_str(
        r#"# Prevent double activation
if [ -n "${VX_ACTIVE:-}" ]; then
    echo "Warning: vx environment is already active. Run 'vx_deactivate' first." >&2
    return 1 2>/dev/null || exit 1
fi

"#,
    );

    // Save current environment
    script.push_str(
        r#"# Save current environment
export _OLD_VX_PATH="$PATH"
export _OLD_VX_PS1="${PS1:-}"

"#,
    );

    // Update PATH
    if !config.path_entries.is_empty() {
        let paths = config.path_entries.join(":");
        script.push_str(&format!(
            "# Add tool paths\nexport PATH=\"{}:$PATH\"\n\n",
            paths
        ));
    }

    // Set VX environment marker
    script.push_str("# VX environment marker\n");
    script.push_str("export VX_ACTIVE=1\n");

    // Set project name if available
    if let Some(name) = &config.name {
        let escaped = escape_single_quoted(name);
        script.push_str(&format!("export VX_PROJECT_NAME='{}'\n", escaped));
    }

    // Set custom environment variables
    if !config.env_vars.is_empty() {
        script.push_str("\n# Custom environment variables\n");
        for (key, value) in &config.env_vars {
            if key == "PATH" {
                continue; // PATH is handled separately
            }
            let escaped = escape_single_quoted(value);
            script.push_str(&format!("export {}='{}'\n", key, escaped));
        }
    }

    // Define aliases
    if !config.aliases.is_empty() {
        script.push_str("\n# Shell aliases\n");
        for (name, command) in &config.aliases {
            let escaped = escape_single_quoted(command);
            script.push_str(&format!("alias {}='{}'\n", name, escaped));
        }
    }

    // Update prompt
    let prompt_prefix = config.prompt_prefix();
    script.push_str(&format!(
        r#"
# Update prompt
export PS1="{} ${{_OLD_VX_PS1:-\\w\\$ }}"

# Type 'vx_deactivate' to exit the vx environment
"#,
        prompt_prefix
    ));

    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_single_quoted() {
        assert_eq!(escape_single_quoted("test"), "test");
        assert_eq!(escape_single_quoted("it's"), "it'\\''s");
        assert_eq!(escape_single_quoted("don't"), "don'\\''t");
    }

    #[test]
    fn test_generate_script_basic() {
        let env = HashMap::new();
        let script = generate_script("echo hello", &env);

        assert!(script.contains("#!/usr/bin/env bash"));
        assert!(script.contains("set -euo pipefail"));
        assert!(script.contains("echo hello"));
    }

    #[test]
    fn test_generate_script_with_env() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());

        let script = generate_script("echo $FOO", &env);
        assert!(script.contains("export FOO='bar'"));
    }

    #[test]
    fn test_escape_single_quotes() {
        let mut env = HashMap::new();
        env.insert("MSG".to_string(), "It's a test".to_string());

        let script = generate_script("echo", &env);
        assert!(script.contains("It'\\''s a test"));
    }

    #[test]
    fn test_activation_script() {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/usr/local/bin".to_string());

        let script = generate_activation_script(&env);
        assert!(script.contains("export PATH='/usr/local/bin'"));
        assert!(script.contains("# vx environment activation script"));
    }

    #[test]
    fn test_full_activation_script_basic() {
        let config = ActivationConfig::new("my-project");
        let script = generate_full_activation_script(&config);

        // Check header
        assert!(script.contains("# vx environment activation script for Bash/Zsh"));

        // Check deactivate function
        assert!(script.contains("vx_deactivate()"));
        assert!(script.contains("export PATH=\"$_OLD_VX_PATH\""));
        assert!(script.contains("export PS1=\"$_OLD_VX_PS1\""));
        assert!(script.contains("unset VX_ACTIVE"));

        // Check double activation prevention
        assert!(script.contains("if [ -n \"${VX_ACTIVE:-}\" ]"));

        // Check environment saving
        assert!(script.contains("export _OLD_VX_PATH=\"$PATH\""));
        assert!(script.contains("export _OLD_VX_PS1=\"${PS1:-}\""));

        // Check VX marker
        assert!(script.contains("export VX_ACTIVE=1"));
        assert!(script.contains("export VX_PROJECT_NAME='my-project'"));

        // Check prompt update
        assert!(script.contains("(my-project[vx])"));
    }

    #[test]
    fn test_full_activation_script_with_paths() {
        let config = ActivationConfig::new("test")
            .with_path("/usr/local/bin")
            .with_path("/opt/tools/bin");

        let script = generate_full_activation_script(&config);
        assert!(script.contains("export PATH=\"/usr/local/bin:/opt/tools/bin:$PATH\""));
    }

    #[test]
    fn test_full_activation_script_with_env_vars() {
        let config = ActivationConfig::new("test")
            .with_env("NODE_ENV", "development")
            .with_env("DEBUG", "true");

        let script = generate_full_activation_script(&config);
        assert!(script.contains("export NODE_ENV='development'"));
        assert!(script.contains("export DEBUG='true'"));
    }

    #[test]
    fn test_full_activation_script_with_aliases() {
        let config = ActivationConfig::new("test")
            .with_alias("ll", "ls -la")
            .with_alias("gs", "git status");

        let script = generate_full_activation_script(&config);
        assert!(script.contains("alias ll='ls -la'"));
        assert!(script.contains("alias gs='git status'"));

        // Check aliases are unset in deactivate
        assert!(script.contains("unalias ll"));
        assert!(script.contains("unalias gs"));
    }

    #[test]
    fn test_full_activation_script_custom_prompt() {
        let config = ActivationConfig::new("myenv").with_prompt_format("[{name}]");

        let script = generate_full_activation_script(&config);
        assert!(script.contains("[myenv]"));
    }

    #[test]
    fn test_full_activation_script_special_chars() {
        let config = ActivationConfig::new("test").with_env("MSG", "It's a \"test\"");

        let script = generate_full_activation_script(&config);
        // Single quotes should be escaped
        assert!(script.contains("It'\\''s a \"test\""));
    }
}
