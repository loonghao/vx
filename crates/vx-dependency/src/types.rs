//! Core types for dependency resolution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specification for a tool and its dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Tool name
    pub name: String,
    /// Tool version (if specific version is required)
    pub version: Option<String>,
    /// List of dependencies
    pub dependencies: Vec<DependencySpec>,
    /// Tool metadata
    pub metadata: HashMap<String, String>,
    /// Whether this tool supports auto-installation
    pub auto_installable: bool,
    /// Installation priority (higher = install first)
    pub priority: i32,
}

/// Specification for a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencySpec {
    /// Name of the dependency tool
    pub tool_name: String,
    /// Version constraint
    pub version_constraint: Option<VersionConstraint>,
    /// Dependency type
    pub dependency_type: DependencyType,
    /// Human-readable description
    pub description: String,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Platform-specific constraints
    pub platforms: Vec<String>,
}

/// Type of dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    /// Required at runtime
    Runtime,
    /// Required for building/compilation
    Build,
    /// Required for development
    Development,
    /// Peer dependency (should be provided by user)
    Peer,
    /// Optional enhancement
    Optional,
}

/// Version constraint specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConstraint {
    /// Constraint expression (e.g., ">=1.0.0", "^2.1.0", "~1.2.3")
    pub expression: String,
    /// Whether to allow prerelease versions
    pub allow_prerelease: bool,
}

impl Default for ToolSpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: None,
            dependencies: Vec::new(),
            metadata: HashMap::new(),
            auto_installable: true,
            priority: 0,
        }
    }
}

impl DependencySpec {
    /// Create a required runtime dependency
    pub fn required(tool_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            version_constraint: None,
            dependency_type: DependencyType::Runtime,
            description: description.into(),
            optional: false,
            platforms: vec![],
        }
    }

    /// Create an optional dependency
    pub fn optional(tool_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            version_constraint: None,
            dependency_type: DependencyType::Optional,
            description: description.into(),
            optional: true,
            platforms: vec![],
        }
    }

    /// Create a build-time dependency
    pub fn build(tool_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            version_constraint: None,
            dependency_type: DependencyType::Build,
            description: description.into(),
            optional: false,
            platforms: vec![],
        }
    }

    /// Set version constraint
    pub fn with_version(mut self, constraint: impl Into<String>) -> Self {
        self.version_constraint = Some(VersionConstraint {
            expression: constraint.into(),
            allow_prerelease: false,
        });
        self
    }

    /// Set version constraint with prerelease support
    pub fn with_version_prerelease(mut self, constraint: impl Into<String>) -> Self {
        self.version_constraint = Some(VersionConstraint {
            expression: constraint.into(),
            allow_prerelease: true,
        });
        self
    }

    /// Set platform constraints
    pub fn for_platforms(mut self, platforms: Vec<String>) -> Self {
        self.platforms = platforms;
        self
    }

    /// Check if this dependency applies to the current platform
    pub fn applies_to_platform(&self, platform: &str) -> bool {
        self.platforms.is_empty() || self.platforms.contains(&platform.to_string())
    }
}

impl VersionConstraint {
    /// Create a new version constraint
    pub fn new(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            allow_prerelease: false,
        }
    }

    /// Create a version constraint that allows prerelease versions
    pub fn with_prerelease(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            allow_prerelease: true,
        }
    }

    /// Check if this constraint is satisfied by a version
    pub fn is_satisfied_by(&self, version: &str) -> bool {
        // TODO: Implement proper semantic version matching
        // For now, simple string comparison
        !version.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_spec_creation() {
        let dep = DependencySpec::required("node", "Node.js runtime")
            .with_version(">=16.0.0")
            .for_platforms(vec!["linux".to_string(), "macos".to_string()]);

        assert_eq!(dep.tool_name, "node");
        assert_eq!(dep.dependency_type, DependencyType::Runtime);
        assert!(!dep.optional);
        assert!(dep.applies_to_platform("linux"));
        assert!(!dep.applies_to_platform("windows"));
    }

    #[test]
    fn test_tool_spec_default() {
        let tool = ToolSpec::default();
        assert!(tool.name.is_empty());
        assert!(tool.dependencies.is_empty());
        assert!(tool.auto_installable);
        assert_eq!(tool.priority, 0);
    }

    #[test]
    fn test_version_constraint() {
        let constraint = VersionConstraint::new(">=1.0.0");
        assert_eq!(constraint.expression, ">=1.0.0");
        assert!(!constraint.allow_prerelease);

        let constraint_pre = VersionConstraint::with_prerelease("^2.0.0-beta");
        assert!(constraint_pre.allow_prerelease);
    }

    #[test]
    fn test_dependency_types() {
        let runtime_dep = DependencySpec::required("node", "Runtime dependency");
        assert_eq!(runtime_dep.dependency_type, DependencyType::Runtime);

        let build_dep = DependencySpec::build("gcc", "Build dependency");
        assert_eq!(build_dep.dependency_type, DependencyType::Build);

        let optional_dep = DependencySpec::optional("docker", "Optional dependency");
        assert_eq!(optional_dep.dependency_type, DependencyType::Optional);
        assert!(optional_dep.optional);
    }
}
