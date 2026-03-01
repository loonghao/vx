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
use std::sync::OnceLock;
use vx_versions::{RangeOp, VersionConstraint, VersionRequest};

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
        let parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();

        if parts.is_empty() {
            return true; // Can't parse, assume match
        }

        // Check major version match
        if let Some(major) = self.major
            && parts[0] != major
        {
            return false;
        }

        // Check min version
        if let Some(ref min) = self.min {
            let min_parts: Vec<u32> = min.split('.').filter_map(|s| s.parse().ok()).collect();
            if !version_gte(&parts, &min_parts) {
                return false;
            }
        }

        // Check max version
        if let Some(ref max) = self.max {
            let max_parts: Vec<u32> = max.split('.').filter_map(|s| s.parse().ok()).collect();
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

    /// Load constraints from a `ProviderManifest`.
    ///
    /// Iterates over all runtimes in the manifest and registers their
    /// `[[runtimes.constraints]]` entries into this registry.
    pub fn load_from_manifest(&mut self, manifest: &vx_manifest::ProviderManifest) {
        for runtime_def in &manifest.runtimes {
            if runtime_def.constraints.is_empty() {
                continue;
            }
            let rules: Vec<ConstraintRule> = runtime_def
                .constraints
                .iter()
                .map(|c: &vx_manifest::ConstraintRule| {
                    let mut rule = ConstraintRule::with_manifest_pattern(
                        ManifestVersionPattern::new(c.when.clone()),
                    );
                    for dep in &c.requires {
                        let mut constraint = DependencyConstraint::required(&dep.runtime);
                        let req = VersionRequest::parse(&dep.version);
                        let (min, max) = extract_min_max_from_constraint(&req);
                        if let Some(min_ver) = min {
                            constraint = constraint.min(min_ver);
                        }
                        if let Some(max_ver) = max {
                            constraint = constraint.max(max_ver);
                        }
                        if let Some(ref rec) = dep.recommended {
                            constraint = constraint.recommended(rec);
                        }
                        if let Some(ref rsn) = dep.reason {
                            constraint = constraint.reason(rsn);
                        }
                        if dep.optional {
                            constraint.optional = true;
                        }
                        rule = rule.with_constraint(constraint);
                    }
                    rule
                })
                .collect();
            self.register(runtime_def.name.clone(), rules);
        }
    }

    /// Build a `ConstraintsRegistry` from an iterable of `(name, toml_str)` pairs.
    ///
    /// Each TOML string is parsed as a `ProviderManifest` and its constraints
    /// are loaded into the registry.
    pub fn from_manifest_strings<'a, I>(manifests: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut registry = Self::new();
        for (_name, toml_str) in manifests {
            let manifest = vx_manifest::ProviderManifest::parse(toml_str)
                .map_err(|e| format!("failed to parse manifest: {e}"))?;
            registry.load_from_manifest(&manifest);
        }
        Ok(registry)
    }
}

/// Extract min and max versions from a VersionRequest (local types)
fn extract_min_max_from_constraint(req: &VersionRequest) -> (Option<String>, Option<String>) {
    match &req.constraint {
        VersionConstraint::Range(constraints) => {
            let mut min = None;
            let mut max = None;
            for c in constraints {
                match c.op {
                    RangeOp::Gte | RangeOp::Gt => {
                        min = Some(c.version.to_string());
                    }
                    RangeOp::Lte | RangeOp::Lt => {
                        max = Some(c.version.to_string());
                    }
                    _ => {}
                }
            }
            (min, max)
        }
        VersionConstraint::Caret(v) => {
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

/// A single dependency entry: (runtime, version, recommended, reason, optional)
type DepEntry = (String, String, Option<String>, Option<String>, bool);

/// A single constraint rule entry: (when_pattern, deps)
type ConstraintRuleEntry = (String, Vec<DepEntry>);

/// Build constraint rules from a list of (when, requires) tuples.
///
/// This is the `provider.star`-friendly API: the Starlark layer calls this
/// after evaluating a provider script to register its constraints.
pub fn build_constraint_rules(rules: &[ConstraintRuleEntry]) -> Vec<ConstraintRule> {
    rules
        .iter()
        .map(|(when, deps)| {
            let mut rule =
                ConstraintRule::with_manifest_pattern(ManifestVersionPattern::new(when.clone()));
            for (runtime, version, recommended, reason, optional) in deps {
                let mut constraint = DependencyConstraint::required(runtime);
                let req = VersionRequest::parse(version);
                let (min, max) = extract_min_max_from_constraint(&req);
                if let Some(min_ver) = min {
                    constraint = constraint.min(min_ver);
                }
                if let Some(max_ver) = max {
                    constraint = constraint.max(max_ver);
                }
                if let Some(rec) = recommended {
                    constraint = constraint.recommended(rec);
                }
                if let Some(rsn) = reason {
                    constraint = constraint.reason(rsn);
                }
                if *optional {
                    constraint.optional = true;
                }
                rule = rule.with_constraint(constraint);
            }
            rule
        })
        .collect()
}

/// Global default constraints registry
///
/// Populated at startup by `init_constraints_from_star()` using embedded
/// `provider.star` files. If not initialized, it will remain empty.
pub static DEFAULT_CONSTRAINTS: OnceLock<ConstraintsRegistry> = OnceLock::new();

fn default_registry() -> &'static ConstraintsRegistry {
    DEFAULT_CONSTRAINTS.get_or_init(ConstraintsRegistry::new)
}

/// Get constraints for a runtime version from the default registry
pub fn get_default_constraints(runtime: &str, version: &str) -> Vec<RuntimeDependency> {
    default_registry().get_constraints(runtime, version)
}

/// Initialize the global constraints registry from provider.star constraint data.
///
/// `rules` is a list of `(runtime_name, constraint_rules)` pairs produced by
/// evaluating all embedded `provider.star` files.
///
/// If already initialized, this is a no-op (idempotent).
pub fn init_constraints_from_star(rules: Vec<(String, Vec<ConstraintRule>)>) -> Result<(), String> {
    let mut registry = ConstraintsRegistry::new();
    for (runtime, constraint_rules) in rules {
        registry.register(runtime, constraint_rules);
    }
    if DEFAULT_CONSTRAINTS.set(registry).is_err() {
        // Already initialized; treat as success for idempotency
    }
    Ok(())
}

/// Initialize the global constraints registry with embedded manifests
///
/// **Deprecated**: use `init_constraints_from_star` instead.
/// Kept temporarily for backward compatibility during the provider.toml → provider.star migration.
#[deprecated(note = "Use init_constraints_from_star instead")]
pub fn init_constraints_from_manifests<'a, I>(_manifests: I) -> Result<(), String>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    // No-op: constraints are now loaded from provider.star
    Ok(())
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
    fn test_manifest_version_pattern() {
        let pattern = ManifestVersionPattern::new("^1");
        assert!(pattern.matches("1.0.0"));
        assert!(pattern.matches("1.22.22"));
        assert!(!pattern.matches("2.0.0"));
    }

    #[test]
    fn test_constraint_rule_matches() {
        let rule = ConstraintRule::with_manifest_pattern(ManifestVersionPattern::new(">=12, <23"))
            .with_constraint(
                DependencyConstraint::required("node")
                    .min("12.0.0")
                    .max("23.0.0"),
            );
        assert!(rule.matches("12.0.0"));
        assert!(rule.matches("20.0.0"));
        assert!(!rule.matches("11.0.0"));
        assert!(!rule.matches("23.0.0"));
    }

    #[test]
    fn test_no_constraints() {
        let registry = ConstraintsRegistry::new();
        let deps = registry.get_constraints("unknown-runtime", "1.0.0");
        assert!(deps.is_empty());
    }
}
