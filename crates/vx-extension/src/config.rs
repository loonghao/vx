//! Extension configuration parsing (vx-extension.toml)

use crate::error::{ExtensionError, ExtensionResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use vx_args::{ArgDef, ArgType};

/// Extension configuration from vx-extension.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    /// Extension metadata
    pub extension: ExtensionMetadata,
    /// Runtime requirements
    #[serde(default)]
    pub runtime: RuntimeRequirement,
    /// Entrypoint configuration
    #[serde(default)]
    pub entrypoint: EntrypointConfig,
    /// Command definitions
    #[serde(default)]
    pub commands: HashMap<String, CommandConfig>,
    /// Hook definitions (future)
    #[serde(default)]
    pub hooks: HashMap<String, String>,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Configuration inheritance
    #[serde(default)]
    pub extends: Option<String>,
}

/// Extension metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMetadata {
    /// Extension name
    pub name: String,
    /// Version string
    #[serde(default = "default_version")]
    pub version: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Extension type
    #[serde(default, rename = "type")]
    pub extension_type: ExtensionType,
    /// Author(s)
    #[serde(default)]
    pub authors: Vec<String>,
    /// License
    #[serde(default)]
    pub license: Option<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Extension type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionType {
    /// Command extension - provides CLI commands
    #[default]
    Command,
    /// Hook extension - executes at lifecycle events
    Hook,
    /// Provider extension - provides runtime support
    Provider,
}

impl std::fmt::Display for ExtensionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionType::Command => write!(f, "command"),
            ExtensionType::Hook => write!(f, "hook"),
            ExtensionType::Provider => write!(f, "provider"),
        }
    }
}

/// Runtime requirement specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeRequirement {
    /// Runtime requirement string (e.g., "python >= 3.10", "node >= 18")
    #[serde(default)]
    pub requires: Option<String>,
    /// Additional dependencies to install
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl RuntimeRequirement {
    /// Parse the runtime name from the requires string
    pub fn runtime_name(&self) -> Option<&str> {
        self.requires
            .as_ref()
            .map(|s| s.split_whitespace().next().unwrap_or(s.as_str()))
    }

    /// Parse the version constraint from the requires string
    pub fn version_constraint(&self) -> Option<&str> {
        self.requires.as_ref().and_then(|s| {
            let parts: Vec<&str> = s.splitn(2, char::is_whitespace).collect();
            if parts.len() > 1 {
                Some(parts[1].trim())
            } else {
                None
            }
        })
    }
}

/// Entrypoint configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntrypointConfig {
    /// Main script file
    #[serde(default)]
    pub main: Option<String>,
    /// Default arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Argument definitions
    #[serde(default)]
    pub arguments: Vec<ArgumentDef>,
}

/// Argument definition in TOML format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentDef {
    /// Argument name
    pub name: String,
    /// Argument type (string, flag, array, number)
    #[serde(default, rename = "type")]
    pub arg_type: String,
    /// Whether the argument is required
    #[serde(default)]
    pub required: bool,
    /// Default value
    #[serde(default)]
    pub default: Option<String>,
    /// Valid choices
    #[serde(default)]
    pub choices: Vec<String>,
    /// Environment variable to read from
    #[serde(default)]
    pub env: Option<String>,
    /// Short flag (single character)
    #[serde(default)]
    pub short: Option<String>,
    /// Help text
    #[serde(default)]
    pub help: Option<String>,
    /// Validation pattern (regex)
    #[serde(default)]
    pub pattern: Option<String>,
    /// Whether this is a positional argument
    #[serde(default)]
    pub positional: bool,
}

impl ArgumentDef {
    /// Convert to vx_args::ArgDef
    pub fn to_arg_def(&self) -> ArgDef {
        let mut arg = ArgDef::new(&self.name);

        // Set type
        arg = match self.arg_type.as_str() {
            "flag" | "bool" | "boolean" => arg.arg_type(ArgType::Flag),
            "array" | "list" => arg.arg_type(ArgType::Array),
            "number" | "int" | "float" => arg.arg_type(ArgType::Number),
            _ => arg.arg_type(ArgType::String),
        };

        // Set other properties
        arg = arg.required(self.required);

        if let Some(ref default) = self.default {
            arg = arg.default(default);
        }

        if !self.choices.is_empty() {
            arg = arg.choices(self.choices.clone());
        }

        if let Some(ref env) = self.env {
            arg = arg.env(env);
        }

        if let Some(ref short) = self.short
            && let Some(c) = short.chars().next()
        {
            arg = arg.short(c);
        }

        if let Some(ref help) = self.help {
            arg = arg.help(help);
        }

        if let Some(ref pattern) = self.pattern {
            arg = arg.pattern(pattern);
        }

        arg = arg.positional(self.positional);

        arg
    }
}

/// Command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    /// Command description
    #[serde(default)]
    pub description: String,
    /// Script file to execute
    pub script: String,
    /// Default arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Argument definitions
    #[serde(default)]
    pub arguments: Vec<ArgumentDef>,
    /// Command-specific environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl ExtensionConfig {
    /// Load extension config from a file
    pub fn from_file(path: &Path) -> ExtensionResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ExtensionError::config_not_found(path)
            } else {
                ExtensionError::io(
                    format!("Failed to read extension config: {}", e),
                    Some(path.to_path_buf()),
                    e,
                )
            }
        })?;
        Self::parse(&content, Some(path))
    }

    /// Parse extension config from a string
    pub fn parse(content: &str, path: Option<&Path>) -> ExtensionResult<Self> {
        toml::from_str(content).map_err(|e| {
            if let Some(p) = path {
                ExtensionError::config_invalid(p, &e)
            } else {
                ExtensionError::ConfigInvalid {
                    path: std::path::PathBuf::from("<string>"),
                    reason: e.message().to_string(),
                    line: e.span().map(|s| s.start),
                    column: None,
                }
            }
        })
    }

    /// Get the script for a subcommand
    pub fn get_command_script(&self, subcommand: &str) -> Option<&CommandConfig> {
        self.commands.get(subcommand)
    }

    /// Get the main entrypoint script
    pub fn get_main_script(&self) -> Option<&str> {
        self.entrypoint.main.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let toml = r#"
[extension]
name = "test-extension"
version = "1.0.0"
description = "A test extension"
type = "command"

[runtime]
requires = "python >= 3.10"

[entrypoint]
main = "main.py"

[commands.hello]
description = "Say hello"
script = "hello.py"
args = ["--verbose"]
"#;

        let config = ExtensionConfig::parse(toml, None).unwrap();
        assert_eq!(config.extension.name, "test-extension");
        assert_eq!(config.extension.version, "1.0.0");
        assert_eq!(config.extension.extension_type, ExtensionType::Command);
        assert_eq!(config.runtime.runtime_name(), Some("python"));
        assert_eq!(config.runtime.version_constraint(), Some(">= 3.10"));
        assert!(config.commands.contains_key("hello"));
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
[extension]
name = "minimal"
"#;

        let config = ExtensionConfig::parse(toml, None).unwrap();
        assert_eq!(config.extension.name, "minimal");
        assert_eq!(config.extension.version, "0.1.0");
        assert_eq!(config.extension.extension_type, ExtensionType::Command);
    }

    #[test]
    fn test_runtime_parsing() {
        let req = RuntimeRequirement {
            requires: Some("node >= 18.0.0".to_string()),
            dependencies: vec![],
        };
        assert_eq!(req.runtime_name(), Some("node"));
        assert_eq!(req.version_constraint(), Some(">= 18.0.0"));
    }

    #[test]
    fn test_parse_invalid_toml() {
        let invalid_toml = r#"
[extension
name = "broken"
"#;
        let result = ExtensionConfig::parse(invalid_toml, None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ExtensionError::ConfigInvalid { .. }));
    }
}
