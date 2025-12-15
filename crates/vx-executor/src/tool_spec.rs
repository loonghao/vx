//! Tool specification and dependency definitions
//!
//! This module defines the structure for tool specifications including
//! their dependencies, aliases, and installation requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specification for a tool including its dependencies and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Primary tool name (e.g., "npm", "cargo", "uvx")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Alternative names for this tool (e.g., "nodejs" for "node")
    pub aliases: Vec<String>,

    /// Runtime dependencies required to execute this tool
    pub dependencies: Vec<RuntimeDependency>,

    /// The actual executable name (may differ from tool name)
    /// e.g., "uvx" might execute as "uv tool run"
    pub executable: Option<String>,

    /// Command prefix to add before user arguments
    /// e.g., uvx adds ["tool", "run"] prefix
    pub command_prefix: Vec<String>,

    /// Environment variables to set when executing
    pub env_vars: HashMap<String, String>,

    /// Whether this tool can be auto-installed
    pub auto_installable: bool,

    /// Installation priority (higher = install first)
    pub priority: i32,

    /// Ecosystem this tool belongs to
    pub ecosystem: Ecosystem,
}

/// Runtime dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDependency {
    /// Name of the required tool
    pub tool_name: String,

    /// Minimum version required (semver constraint)
    pub min_version: Option<String>,

    /// Whether this dependency is required or optional
    pub required: bool,

    /// Reason for this dependency
    pub reason: String,

    /// The tool that provides this dependency
    /// e.g., "npm" is provided by "node" bundle
    pub provided_by: Option<String>,
}

/// Ecosystem categorization for tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Ecosystem {
    /// Node.js ecosystem (npm, yarn, pnpm, bun)
    Node,
    /// Python ecosystem (uv, pip, poetry)
    Python,
    /// Rust ecosystem (cargo, rustup)
    Rust,
    /// Go ecosystem (go)
    Go,
    /// Generic/standalone tools
    #[default]
    Generic,
}

impl ToolSpec {
    /// Create a new tool specification
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            aliases: Vec::new(),
            dependencies: Vec::new(),
            executable: None,
            command_prefix: Vec::new(),
            env_vars: HashMap::new(),
            auto_installable: true,
            priority: 0,
            ecosystem: Ecosystem::Generic,
        }
    }

    /// Add an alias for this tool
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add multiple aliases
    pub fn with_aliases(mut self, aliases: Vec<&str>) -> Self {
        self.aliases.extend(aliases.into_iter().map(String::from));
        self
    }

    /// Add a required runtime dependency
    pub fn with_dependency(mut self, dep: RuntimeDependency) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Set the actual executable name
    pub fn with_executable(mut self, exe: impl Into<String>) -> Self {
        self.executable = Some(exe.into());
        self
    }

    /// Set command prefix
    pub fn with_command_prefix(mut self, prefix: Vec<&str>) -> Self {
        self.command_prefix = prefix.into_iter().map(String::from).collect();
        self
    }

    /// Set ecosystem
    pub fn with_ecosystem(mut self, ecosystem: Ecosystem) -> Self {
        self.ecosystem = ecosystem;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Get the executable name (defaults to tool name)
    pub fn get_executable(&self) -> &str {
        self.executable.as_deref().unwrap_or(&self.name)
    }

    /// Check if this tool matches a given name or alias
    pub fn matches(&self, name: &str) -> bool {
        self.name == name || self.aliases.iter().any(|a| a == name)
    }

    /// Get all required dependencies
    pub fn required_dependencies(&self) -> Vec<&RuntimeDependency> {
        self.dependencies.iter().filter(|d| d.required).collect()
    }
}

impl RuntimeDependency {
    /// Create a required dependency
    pub fn required(tool_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            min_version: None,
            required: true,
            reason: reason.into(),
            provided_by: None,
        }
    }

    /// Create an optional dependency
    pub fn optional(tool_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            min_version: None,
            required: false,
            reason: reason.into(),
            provided_by: None,
        }
    }

    /// Set minimum version constraint
    pub fn with_min_version(mut self, version: impl Into<String>) -> Self {
        self.min_version = Some(version.into());
        self
    }

    /// Set the provider tool
    pub fn provided_by(mut self, provider: impl Into<String>) -> Self {
        self.provided_by = Some(provider.into());
        self
    }
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ecosystem::Node => write!(f, "node"),
            Ecosystem::Python => write!(f, "python"),
            Ecosystem::Rust => write!(f, "rust"),
            Ecosystem::Go => write!(f, "go"),
            Ecosystem::Generic => write!(f, "generic"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_spec_creation() {
        let spec = ToolSpec::new("npm", "Node.js package manager")
            .with_alias("npm-cli")
            .with_ecosystem(Ecosystem::Node)
            .with_dependency(RuntimeDependency::required(
                "node",
                "npm requires Node.js runtime",
            ));

        assert_eq!(spec.name, "npm");
        assert!(spec.matches("npm"));
        assert!(spec.matches("npm-cli"));
        assert!(!spec.matches("yarn"));
        assert_eq!(spec.ecosystem, Ecosystem::Node);
        assert_eq!(spec.required_dependencies().len(), 1);
    }

    #[test]
    fn test_runtime_dependency() {
        let dep = RuntimeDependency::required("node", "Required runtime")
            .with_min_version(">=16.0.0")
            .provided_by("node-bundle");

        assert!(dep.required);
        assert_eq!(dep.tool_name, "node");
        assert_eq!(dep.min_version, Some(">=16.0.0".to_string()));
        assert_eq!(dep.provided_by, Some("node-bundle".to_string()));
    }
}
