//! Runtime specification and dependency definitions
//!
//! This module defines the structure for runtime specifications including
//! their dependencies, aliases, and installation requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specification for a runtime including its dependencies and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSpec {
    /// Primary runtime name (e.g., "npm", "cargo", "uvx")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Alternative names for this runtime (e.g., "nodejs" for "node")
    pub aliases: Vec<String>,

    /// Runtime dependencies required to execute this runtime
    pub dependencies: Vec<RuntimeDependency>,

    /// The actual executable name (may differ from runtime name)
    /// e.g., "uvx" might execute as "uv tool run"
    pub executable: Option<String>,

    /// Command prefix to add before user arguments
    /// e.g., uvx adds ["tool", "run"] prefix
    pub command_prefix: Vec<String>,

    /// Environment variables to set when executing
    pub env_vars: HashMap<String, String>,

    /// Advanced environment configuration
    pub env_config: Option<vx_manifest::EnvConfig>,

    /// Whether this runtime can be auto-installed
    pub auto_installable: bool,

    /// Installation priority (higher = install first)
    pub priority: i32,

    /// Ecosystem this runtime belongs to
    pub ecosystem: Ecosystem,
}

/// Runtime dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDependency {
    /// Name of the required runtime
    pub runtime_name: String,

    /// Minimum version required (semver constraint)
    pub min_version: Option<String>,

    /// Maximum version allowed (semver constraint)
    /// Use this to exclude incompatible versions
    /// e.g., yarn 1.x may not work well with Node.js 23+
    pub max_version: Option<String>,

    /// Recommended version for this dependency
    /// If the dependency needs to be installed, use this version
    pub recommended_version: Option<String>,

    /// Whether this dependency is required or optional
    pub required: bool,

    /// Reason for this dependency
    pub reason: String,

    /// The provider that provides this dependency
    /// e.g., "npm" is provided by "node" provider
    pub provided_by: Option<String>,
}

/// Ecosystem categorization for runtimes
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
    /// Java ecosystem (java, javac, jar)
    Java,
    /// Generic/standalone runtimes
    #[default]
    Generic,
}

impl RuntimeSpec {
    /// Create a new runtime specification
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            aliases: Vec::new(),
            dependencies: Vec::new(),
            executable: None,
            command_prefix: Vec::new(),
            env_vars: HashMap::new(),
            env_config: None,
            auto_installable: true,
            priority: 0,
            ecosystem: Ecosystem::Generic,
        }
    }

    /// Add an alias for this runtime
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

    /// Get the executable name (defaults to runtime name)
    pub fn get_executable(&self) -> &str {
        self.executable.as_deref().unwrap_or(&self.name)
    }

    /// Check if this runtime matches a given name or alias
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
    pub fn required(runtime_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            runtime_name: runtime_name.into(),
            min_version: None,
            max_version: None,
            recommended_version: None,
            required: true,
            reason: reason.into(),
            provided_by: None,
        }
    }

    /// Create an optional dependency
    pub fn optional(runtime_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            runtime_name: runtime_name.into(),
            min_version: None,
            max_version: None,
            recommended_version: None,
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

    /// Set maximum version constraint
    ///
    /// Use this to exclude incompatible versions.
    /// For example, yarn 1.x may have issues with Node.js 23+
    pub fn with_max_version(mut self, version: impl Into<String>) -> Self {
        self.max_version = Some(version.into());
        self
    }

    /// Set recommended version
    ///
    /// If the dependency needs to be installed, this version will be used
    pub fn with_recommended_version(mut self, version: impl Into<String>) -> Self {
        self.recommended_version = Some(version.into());
        self
    }

    /// Set the provider
    pub fn provided_by(mut self, provider: impl Into<String>) -> Self {
        self.provided_by = Some(provider.into());
        self
    }

    /// Check if a version satisfies this dependency's constraints
    pub fn is_version_compatible(&self, version: &str) -> bool {
        // Parse version for comparison
        let parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();

        if parts.is_empty() {
            return true; // Can't parse, assume compatible
        }

        // Check minimum version
        if let Some(ref min) = self.min_version {
            let min_parts: Vec<u32> = min.split('.').filter_map(|s| s.parse().ok()).collect();
            if !Self::version_gte(&parts, &min_parts) {
                return false;
            }
        }

        // Check maximum version
        if let Some(ref max) = self.max_version {
            let max_parts: Vec<u32> = max.split('.').filter_map(|s| s.parse().ok()).collect();
            if !Self::version_lte(&parts, &max_parts) {
                return false;
            }
        }

        true
    }

    /// Compare version parts: a >= b
    fn version_gte(a: &[u32], b: &[u32]) -> bool {
        for i in 0..std::cmp::max(a.len(), b.len()) {
            let av = a.get(i).copied().unwrap_or(0);
            let bv = b.get(i).copied().unwrap_or(0);
            if av > bv {
                return true;
            }
            if av < bv {
                return false;
            }
        }
        true // Equal
    }

    /// Compare version parts: a <= b
    fn version_lte(a: &[u32], b: &[u32]) -> bool {
        for i in 0..std::cmp::max(a.len(), b.len()) {
            let av = a.get(i).copied().unwrap_or(0);
            let bv = b.get(i).copied().unwrap_or(0);
            if av < bv {
                return true;
            }
            if av > bv {
                return false;
            }
        }
        true // Equal
    }
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ecosystem::Node => write!(f, "node"),
            Ecosystem::Python => write!(f, "python"),
            Ecosystem::Rust => write!(f, "rust"),
            Ecosystem::Go => write!(f, "go"),
            Ecosystem::Java => write!(f, "java"),
            Ecosystem::Generic => write!(f, "generic"),
        }
    }
}
