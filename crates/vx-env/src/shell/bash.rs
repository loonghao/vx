//! Bash script generator

use std::collections::HashMap;

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
        // Escape single quotes: replace ' with '\''
        let escaped_value = value.replace('\'', "'\\''");
        script.push_str(&format!("export {}='{}'\n", key, escaped_value));
    }

    // Execute the command
    script.push_str(&format!("\n{}\n", cmd));

    script
}

/// Generate an activation script for interactive shell use
///
/// Unlike `generate_script`, this doesn't execute a command but sets up
/// the environment for interactive use.
pub fn generate_activation_script(env_vars: &HashMap<String, String>) -> String {
    let mut script = String::new();

    script.push_str("# vx environment activation script\n");
    script.push_str("# Source this file: source <(vx env activate)\n\n");

    for (key, value) in env_vars {
        let escaped_value = value.replace('\'', "'\\''");
        script.push_str(&format!("export {}='{}'\n", key, escaped_value));
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
}
