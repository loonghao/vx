//! Shell-specific script generators
//!
//! This module provides platform-specific script generation for different shells.
//!
//! # Virtual Environment Design
//!
//! The activation scripts follow the same design patterns as Python's venv, uv venv,
//! and conda environments:
//!
//! 1. **Save original environment** - Store PATH, PS1/prompt before modification
//! 2. **Modify environment** - Add tool paths, set environment variables
//! 3. **Update prompt** - Show environment name in shell prompt
//! 4. **Provide deactivate** - Function to restore original environment
//! 5. **Prevent double activation** - Check if already activated

pub mod bash;
pub mod cmd;
pub mod powershell;

use std::collections::HashMap;

/// Configuration for generating activation scripts
///
/// This struct provides all the information needed to generate a complete
/// virtual environment activation script, similar to venv or conda.
#[derive(Debug, Clone, Default)]
pub struct ActivationConfig {
    /// Project or environment name (displayed in prompt)
    pub name: Option<String>,

    /// Additional PATH entries to prepend
    pub path_entries: Vec<String>,

    /// Environment variables to set
    pub env_vars: HashMap<String, String>,

    /// Custom prompt format (optional)
    /// Template variables: {name}
    /// Default: "({name}[vx])"
    pub prompt_format: Option<String>,

    /// Shell aliases to define
    pub aliases: HashMap<String, String>,
}

impl ActivationConfig {
    /// Create a new activation config with a project name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Add a PATH entry
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path_entries.push(path.into());
        self
    }

    /// Add multiple PATH entries
    pub fn with_paths(mut self, paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.path_entries
            .extend(paths.into_iter().map(|p| p.into()));
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables
    pub fn with_envs(mut self, vars: HashMap<String, String>) -> Self {
        self.env_vars.extend(vars);
        self
    }

    /// Set custom prompt format
    pub fn with_prompt_format(mut self, format: impl Into<String>) -> Self {
        self.prompt_format = Some(format.into());
        self
    }

    /// Add a shell alias
    pub fn with_alias(mut self, name: impl Into<String>, command: impl Into<String>) -> Self {
        self.aliases.insert(name.into(), command.into());
        self
    }

    /// Get the formatted prompt prefix
    pub fn prompt_prefix(&self) -> String {
        let name = self.name.as_deref().unwrap_or("vx");
        match &self.prompt_format {
            Some(format) => format.replace("{name}", name),
            None => format!("({}[vx])", name),
        }
    }
}

/// Shell type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    /// Bash shell (Linux/macOS default)
    Bash,
    /// PowerShell (Windows default, also available on Linux/macOS)
    PowerShell,
    /// Windows Command Prompt
    Cmd,
    /// POSIX sh (fallback)
    Sh,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
}

impl Shell {
    /// Detect the current shell from environment
    pub fn detect() -> Self {
        if cfg!(windows) {
            // On Windows, prefer PowerShell
            Shell::PowerShell
        } else {
            // On Unix, check SHELL environment variable
            if let Ok(shell) = std::env::var("SHELL") {
                if shell.contains("zsh") {
                    Shell::Zsh
                } else if shell.contains("fish") {
                    Shell::Fish
                } else if shell.contains("bash") {
                    Shell::Bash
                } else {
                    Shell::Sh
                }
            } else {
                Shell::Bash
            }
        }
    }

    /// Get the script file extension for this shell
    pub fn extension(&self) -> &'static str {
        match self {
            Shell::Bash | Shell::Sh | Shell::Zsh => "sh",
            Shell::PowerShell => "ps1",
            Shell::Cmd => "bat",
            Shell::Fish => "fish",
        }
    }

    /// Get the shell executable name
    pub fn executable(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Sh => "sh",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "pwsh",
            Shell::Cmd => "cmd",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_extension() {
        assert_eq!(Shell::Bash.extension(), "sh");
        assert_eq!(Shell::PowerShell.extension(), "ps1");
        assert_eq!(Shell::Cmd.extension(), "bat");
    }

    #[test]
    fn test_shell_executable() {
        assert_eq!(Shell::Bash.executable(), "bash");
        assert_eq!(Shell::PowerShell.executable(), "pwsh");
    }

    #[test]
    fn test_activation_config_new() {
        let config = ActivationConfig::new("my-project");
        assert_eq!(config.name, Some("my-project".to_string()));
        assert!(config.path_entries.is_empty());
        assert!(config.env_vars.is_empty());
    }

    #[test]
    fn test_activation_config_builder() {
        let config = ActivationConfig::new("test")
            .with_path("/usr/local/bin")
            .with_env("FOO", "bar")
            .with_alias("ll", "ls -la");

        assert_eq!(config.path_entries, vec!["/usr/local/bin"]);
        assert_eq!(config.env_vars.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(config.aliases.get("ll"), Some(&"ls -la".to_string()));
    }

    #[test]
    fn test_activation_config_prompt_prefix() {
        let config = ActivationConfig::new("my-project");
        assert_eq!(config.prompt_prefix(), "(my-project[vx])");

        let custom = ActivationConfig::new("test").with_prompt_format("[{name}]");
        assert_eq!(custom.prompt_prefix(), "[test]");
    }

    #[test]
    fn test_activation_config_default_prompt() {
        let config = ActivationConfig::default();
        assert_eq!(config.prompt_prefix(), "(vx[vx])");
    }
}
