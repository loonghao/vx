//! Runtime dependency constraints registry
//!
//! This module provides a flexible, configuration-based system for defining
//! runtime dependency constraints. Instead of hardcoding constraints in each
//! Provider, constraints are defined declaratively and can be overridden.
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_runtime::constraints::{ConstraintsRegistry, RuntimeConstraint};
//!
//! let registry = ConstraintsRegistry::default();
//!
//! // Get constraints for yarn@1.22.22
//! if let Some(constraints) = registry.get_constraints("yarn", "1.22.22") {
//!     for constraint in constraints {
//!         println!("{} requires {} {}", constraint.runtime, constraint.dependency, constraint.version_range);
//!     }
//! }
//! ```

use crate::RuntimeDependency;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Version range pattern for matching runtime versions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionPattern {
    /// Minimum version (inclusive), e.g., "1.0.0"
    pub min: Option<String>,
    /// Maximum version (exclusive for major), e.g., "2.0.0"
    pub max: Option<String>,
    /// Exact major version match, e.g., "1" matches 1.x.x
    pub major: Option<u32>,
}

impl VersionPattern {
    /// Create a pattern matching a specific major version
    pub fn major(major: u32) -> Self {
        Self {
            min: None,
            max: None,
            major: Some(major),
        }
    }

    /// Create a pattern matching a version range
    pub fn range(min: impl Into<String>, max: impl Into<String>) -> Self {
        Self {
            min: Some(min.into()),
            max: Some(max.into()),
            major: None,
        }
    }

    /// Create a pattern matching all versions
    pub fn all() -> Self {
        Self {
            min: None,
            max: None,
            major: None,
        }
    }

    /// Check if a version matches this pattern
    pub fn matches(&self, version: &str) -> bool {
        let parts: Vec<u32> = version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        if parts.is_empty() {
            return true; // Can't parse, assume match
        }

        // Check major version match
        if let Some(major) = self.major {
            if parts[0] != major {
                return false;
            }
        }

        // Check min version
        if let Some(ref min) = self.min {
            let min_parts: Vec<u32> = min
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();
            if !version_gte(&parts, &min_parts) {
                return false;
            }
        }

        // Check max version
        if let Some(ref max) = self.max {
            let max_parts: Vec<u32> = max
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();
            if !version_lt(&parts, &max_parts) {
                return false;
            }
        }

        true
    }
}

/// Version pattern that uses semver syntax from manifests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestVersionPattern {
    /// The raw semver pattern (e.g., "^1", ">=2, <4", "*")
    pub pattern: String,
}

impl ManifestVersionPattern {
    /// Create a new manifest version pattern
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }

    /// Check if a version matches this pattern using semver semantics
    pub fn matches(&self, version: &str) -> bool {
        use vx_manifest::VersionRequest;

        let req = VersionRequest::parse(&self.pattern);
        req.satisfies(version)
    }
}

/// A single dependency constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConstraint {
    /// The dependency runtime name (e.g., "node")
    pub dependency: String,
    /// Minimum version of the dependency
    pub min_version: Option<String>,
    /// Maximum version of the dependency
    pub max_version: Option<String>,
    /// Recommended version
    pub recommended_version: Option<String>,
    /// Reason for this constraint
    pub reason: Option<String>,
    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

impl DependencyConstraint {
    /// Create a new required dependency constraint
    pub fn required(dependency: impl Into<String>) -> Self {
        Self {
            dependency: dependency.into(),
            min_version: None,
            max_version: None,
            recommended_version: None,
            reason: None,
            optional: false,
        }
    }

    /// Set minimum version
    pub fn min(mut self, version: impl Into<String>) -> Self {
        self.min_version = Some(version.into());
        self
    }

    /// Set maximum version
    pub fn max(mut self, version: impl Into<String>) -> Self {
        self.max_version = Some(version.into());
        self
    }

    /// Set recommended version
    pub fn recommended(mut self, version: impl Into<String>) -> Self {
        self.recommended_version = Some(version.into());
        self
    }

    /// Set reason
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Convert to RuntimeDependency
    pub fn to_runtime_dependency(&self) -> RuntimeDependency {
        let mut dep = RuntimeDependency::required(&self.dependency);
        if let Some(ref min) = self.min_version {
            dep = dep.with_min_version(min);
        }
        if let Some(ref max) = self.max_version {
            dep = dep.with_max_version(max);
        }
        if let Some(ref rec) = self.recommended_version {
            dep = dep.with_recommended_version(rec);
        }
        if let Some(ref reason) = self.reason {
            dep = dep.with_reason(reason);
        }
        if self.optional {
            dep.optional = true;
        }
        dep
    }
}

/// Constraint rule: applies constraints to matching runtime versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintRule {
    /// Version pattern to match (legacy format)
    #[serde(default)]
    pub version_pattern: VersionPattern,
    /// Manifest version pattern (semver format)
    #[serde(default)]
    pub manifest_pattern: Option<ManifestVersionPattern>,
    /// Constraints that apply when pattern matches
    pub constraints: Vec<DependencyConstraint>,
}

impl ConstraintRule {
    /// Create a new constraint rule with legacy pattern
    pub fn new(pattern: VersionPattern) -> Self {
        Self {
            version_pattern: pattern,
            manifest_pattern: None,
            constraints: Vec::new(),
        }
    }

    /// Create a new constraint rule with manifest pattern
    pub fn with_manifest_pattern(pattern: ManifestVersionPattern) -> Self {
        Self {
            version_pattern: VersionPattern::all(),
            manifest_pattern: Some(pattern),
            constraints: Vec::new(),
        }
    }

    /// Add a constraint
    pub fn with_constraint(mut self, constraint: DependencyConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Check if this rule matches a version
    pub fn matches(&self, version: &str) -> bool {
        // Prefer manifest pattern if available
        if let Some(ref mp) = self.manifest_pattern {
            mp.matches(version)
        } else {
            self.version_pattern.matches(version)
        }
    }
}

/// Registry of all runtime constraints
#[derive(Debug, Clone, Default)]
pub struct ConstraintsRegistry {
    /// Runtime name -> list of constraint rules
    rules: HashMap<String, Vec<ConstraintRule>>,
}

impl ConstraintsRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Create registry with built-in constraints
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.load_builtins();
        registry
    }

    /// Register constraint rules for a runtime
    pub fn register(&mut self, runtime: impl Into<String>, rules: Vec<ConstraintRule>) {
        self.rules.insert(runtime.into(), rules);
    }

    /// Get constraints for a specific runtime version
    pub fn get_constraints(&self, runtime: &str, version: &str) -> Vec<RuntimeDependency> {
        let Some(rules) = self.rules.get(runtime) else {
            return Vec::new();
        };

        // Find matching rules and collect constraints
        let mut deps = Vec::new();
        for rule in rules {
            if rule.matches(version) {
                for constraint in &rule.constraints {
                    deps.push(constraint.to_runtime_dependency());
                }
            }
        }

        deps
    }

    /// Check if a runtime has any constraints defined
    pub fn has_constraints(&self, runtime: &str) -> bool {
        self.rules.contains_key(runtime)
    }

    /// Load constraints from a provider manifest
    pub fn load_from_manifest(&mut self, manifest: &vx_manifest::ProviderManifest) {
        for runtime_def in &manifest.runtimes {
            let rules = self.convert_manifest_constraints(&runtime_def.constraints);
            if !rules.is_empty() {
                self.register(&runtime_def.name, rules);
            }
        }
    }

    /// Convert manifest constraints to internal constraint rules
    fn convert_manifest_constraints(
        &self,
        manifest_constraints: &[vx_manifest::ConstraintRule],
    ) -> Vec<ConstraintRule> {
        use vx_manifest::VersionRequest;

        manifest_constraints
            .iter()
            .map(|mc| {
                let mut rule = ConstraintRule::with_manifest_pattern(
                    ManifestVersionPattern::new(mc.when.clone())
                );

                for dep in &mc.requires {
                    let mut constraint = DependencyConstraint::required(&dep.runtime);

                    // Parse version constraint to extract min/max
                    let req = VersionRequest::parse(&dep.version);
                    let (min, max) = Self::extract_min_max_from_constraint(&req);

                    if let Some(min_ver) = min {
                        constraint = constraint.min(min_ver);
                    }
                    if let Some(max_ver) = max {
                        constraint = constraint.max(max_ver);
                    }
                    if let Some(ref rec) = dep.recommended {
                        constraint = constraint.recommended(rec);
                    }
                    if let Some(ref reason) = dep.reason {
                        constraint = constraint.reason(reason);
                    }
                    if dep.optional {
                        constraint.optional = true;
                    }

                    rule = rule.with_constraint(constraint);
                }

                rule
            })
            .collect()
    }

    /// Extract min and max versions from a VersionRequest
    fn extract_min_max_from_constraint(
        req: &vx_manifest::VersionRequest,
    ) -> (Option<String>, Option<String>) {
        use vx_manifest::{RangeOp, VersionConstraint};

        match &req.constraint {
            VersionConstraint::Range(constraints) => {
                let mut min = None;
                let mut max = None;

                for c in constraints {
                    match c.op {
                        RangeOp::Ge | RangeOp::Gt => {
                            min = Some(c.version.to_string());
                        }
                        RangeOp::Le | RangeOp::Lt => {
                            max = Some(c.version.to_string());
                        }
                        _ => {}
                    }
                }

                (min, max)
            }
            VersionConstraint::Caret(v) => {
                // ^1.2.3 means >=1.2.3, <2.0.0
                let min = v.to_string();
                let max = if v.major > 0 {
                    format!("{}.0.0", v.major + 1)
                } else if v.minor > 0 {
                    format!("0.{}.0", v.minor + 1)
                } else {
                    format!("0.0.{}", v.patch + 1)
                };
                (Some(min), Some(max))
            }
            VersionConstraint::Tilde(v) => {
                // ~1.2.3 means >=1.2.3, <1.3.0
                let min = v.to_string();
                let max = format!("{}.{}.0", v.major, v.minor + 1);
                (Some(min), Some(max))
            }
            VersionConstraint::Exact(v) => {
                let ver = v.to_string();
                (Some(ver.clone()), Some(ver))
            }
            _ => (None, None),
        }
    }

    /// Load built-in constraints
    fn load_builtins(&mut self) {
        // Yarn constraints
        self.register("yarn", vec![
            // Yarn 1.x (Classic) requires Node.js 12-22
            ConstraintRule::new(VersionPattern::major(1))
                .with_constraint(
                    DependencyConstraint::required("node")
                        .min("12.0.0")
                        .max("22.99.99")
                        .recommended("20")
                        .reason("Yarn 1.x requires Node.js 12-22 for native module compatibility")
                ),
            // Yarn 2.x-3.x (Berry) requires Node.js 16+
            ConstraintRule::new(VersionPattern::range("2.0.0", "4.0.0"))
                .with_constraint(
                    DependencyConstraint::required("node")
                        .min("16.0.0")
                        .recommended("20")
                        .reason("Yarn 2.x-3.x requires Node.js 16+")
                ),
            // Yarn 4.x requires Node.js 18+
            ConstraintRule::new(VersionPattern::major(4))
                .with_constraint(
                    DependencyConstraint::required("node")
                        .min("18.0.0")
                        .recommended("22")
                        .reason("Yarn 4.x requires Node.js 18+")
                ),
        ]);

        // npm constraints
        self.register("npm", vec![
            // npm 6.x requires Node.js 6-14
            ConstraintRule::new(VersionPattern::major(6))
                .with_constraint(
                    DependencyConstraint::required("node")
                        .min("6.0.0")
                        .max("14.99.99")
                        .recommended("14")
                        .reason("npm 6.x is designed for Node.js 6-14")
                ),
            // npm 7.x-8.x requires Node.js 12+
            ConstraintRule::new(VersionPattern::range("7.0.0", "9.0.0"))
                .with_constraint(
                    DependencyConstraint::required("node")
                        .min("12.0.0")
                        .recommended("18")
                        .reason("npm 7.x-8.x requires Node.js 12+")
                ),
            // npm 9.x+ requires Node.js 14+
            ConstraintRule::new(VersionPattern::range("9.0.0", "99.0.0"))
                .with_constraint(
                    DependencyConstraint::required("node")
                        .min("14.0.0")
                        .recommended("20")
                        .reason("npm 9.x+ requires Node.js 14+")
                ),
        ]);

        // pnpm constraints
        self.register("pnpm", vec![
            // pnpm 7.x requires Node.js 14+
            ConstraintRule::new(VersionPattern::major(7))
                .with_constraint(
                    DependencyConstraint::required("node")
                        .min("14.0.0")
                        .recommended("18")
                        .reason("pnpm 7.x requires Node.js 14+")
                ),
            // pnpm 8.x requires Node.js 16+
            ConstraintRule::new(VersionPattern::major(8))
                .with_constraint(
                    DependencyConstraint::required("node")
                        .min("16.0.0")
                        .recommended("20")
                        .reason("pnpm 8.x requires Node.js 16+")
                ),
            // pnpm 9.x requires Node.js 18+
            ConstraintRule::new(VersionPattern::major(9))
                .with_constraint(
                    DependencyConstraint::required("node")
                        .min("18.0.0")
                        .recommended("22")
                        .reason("pnpm 9.x requires Node.js 18+")
                ),
        ]);

        // npx inherits from npm
        self.register("npx", vec![
            ConstraintRule::new(VersionPattern::all())
                .with_constraint(
                    DependencyConstraint::required("node")
                        .min("12.0.0")
                        .recommended("20")
                        .reason("npx requires Node.js 12+")
                ),
        ]);

        // uvx requires uv
        self.register("uvx", vec![
            ConstraintRule::new(VersionPattern::all())
                .with_constraint(
                    DependencyConstraint::required("uv")
                        .min("0.1.0")
                        .recommended("0.5")
                        .reason("uvx is provided by uv")
                ),
        ]);

        // pip requires python
        self.register("pip", vec![
            ConstraintRule::new(VersionPattern::all())
                .with_constraint(
                    DependencyConstraint::required("python")
                        .min("3.7.0")
                        .recommended("3.12")
                        .reason("pip requires Python 3.7+")
                ),
        ]);
    }
}

/// Global default constraints registry
pub static DEFAULT_CONSTRAINTS: LazyLock<ConstraintsRegistry> = LazyLock::new(|| {
    ConstraintsRegistry::with_builtins()
});

/// Get constraints for a runtime version from the default registry
pub fn get_default_constraints(runtime: &str, version: &str) -> Vec<RuntimeDependency> {
    DEFAULT_CONSTRAINTS.get_constraints(runtime, version)
}

// Helper functions for version comparison
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

fn version_lt(a: &[u32], b: &[u32]) -> bool {
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
    false // Equal means not less than
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_pattern_major() {
        let pattern = VersionPattern::major(1);
        assert!(pattern.matches("1.0.0"));
        assert!(pattern.matches("1.22.22"));
        assert!(pattern.matches("1.99.99"));
        assert!(!pattern.matches("2.0.0"));
        assert!(!pattern.matches("0.9.0"));
    }

    #[test]
    fn test_version_pattern_range() {
        let pattern = VersionPattern::range("2.0.0", "4.0.0");
        assert!(!pattern.matches("1.99.99"));
        assert!(pattern.matches("2.0.0"));
        assert!(pattern.matches("3.5.0"));
        assert!(!pattern.matches("4.0.0")); // max is exclusive
    }

    #[test]
    fn test_yarn_1x_constraints() {
        let registry = ConstraintsRegistry::with_builtins();
        let deps = registry.get_constraints("yarn", "1.22.22");
        
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "node");
        assert_eq!(deps[0].min_version, Some("12.0.0".to_string()));
        assert_eq!(deps[0].max_version, Some("22.99.99".to_string()));
    }

    #[test]
    fn test_yarn_4x_constraints() {
        let registry = ConstraintsRegistry::with_builtins();
        let deps = registry.get_constraints("yarn", "4.0.0");
        
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "node");
        assert_eq!(deps[0].min_version, Some("18.0.0".to_string()));
        assert!(deps[0].max_version.is_none()); // No max for yarn 4.x
    }

    #[test]
    fn test_no_constraints() {
        let registry = ConstraintsRegistry::with_builtins();
        let deps = registry.get_constraints("unknown-runtime", "1.0.0");
        assert!(deps.is_empty());
    }
}
