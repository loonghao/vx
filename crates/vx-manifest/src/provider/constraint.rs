use crate::VersionRequest;
use serde::{Deserialize, Serialize};

/// Constraint rule - defines dependencies for a version range
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConstraintRule {
    /// Version condition (semver syntax)
    /// Examples: "^1", ">=2, <4", "*"
    pub when: String,
    /// Platform condition (optional)
    #[serde(default)]
    pub platform: Option<String>,
    /// Required dependencies
    #[serde(default)]
    pub requires: Vec<DependencyDef>,
    /// Recommended dependencies (optional, not enforced)
    #[serde(default)]
    pub recommends: Vec<DependencyDef>,
}

impl ConstraintRule {
    /// Check if this rule applies to the given version
    pub fn matches(&self, version: &str) -> bool {
        let req = VersionRequest::parse(&self.when);
        req.satisfies(version)
    }

    /// Check if this rule applies to the given platform
    pub fn matches_platform(&self, platform: &str) -> bool {
        match &self.platform {
            Some(p) => p == platform || p == "*",
            None => true, // No platform restriction
        }
    }
}

/// Dependency definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DependencyDef {
    /// Runtime name of the dependency
    pub runtime: String,
    /// Version constraint (semver syntax)
    pub version: String,
    /// Recommended version to install if none available
    #[serde(default)]
    pub recommended: Option<String>,
    /// Reason for this dependency
    #[serde(default)]
    pub reason: Option<String>,
    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

impl DependencyDef {
    /// Check if a version satisfies this dependency constraint
    pub fn satisfies(&self, version: &str) -> bool {
        let req = VersionRequest::parse(&self.version);
        req.satisfies(version)
    }
}
