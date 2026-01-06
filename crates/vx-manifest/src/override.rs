//! Provider manifest override support
//!
//! This module provides types and logic for user-defined constraint overrides.
//! Users can create `.override.toml` files in `~/.vx/providers/` or `<project>/.vx/providers/`
//! to customize dependency constraints without modifying the original provider manifests.
//!
//! # Override File Format
//!
//! ```toml
//! # ~/.vx/providers/yarn.override.toml
//!
//! # Override specific constraints
//! [[constraints]]
//! when = "^1"
//! requires = [
//!     # Company internal uses Node 14-20
//!     { runtime = "node", version = ">=14, <21" }
//! ]
//!
//! # Add additional constraints
//! [[constraints]]
//! when = "*"
//! requires = [
//!     { runtime = "git", version = ">=2.0", optional = true }
//! ]
//! ```

use crate::{ConstraintRule, ProviderManifest, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Provider override configuration
///
/// This is a simplified manifest format that only allows overriding constraints.
/// The provider name is derived from the filename (e.g., `yarn.override.toml` -> `yarn`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProviderOverride {
    /// Constraint overrides for the default runtime
    /// (the runtime with the same name as the provider)
    #[serde(default)]
    pub constraints: Vec<ConstraintRule>,

    /// Runtime-specific constraint overrides
    #[serde(default)]
    pub runtimes: Vec<RuntimeOverride>,
}

/// Runtime-specific override
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeOverride {
    /// Runtime name to override
    pub name: String,
    /// Constraint overrides for this runtime
    #[serde(default)]
    pub constraints: Vec<ConstraintRule>,
}

impl ProviderOverride {
    /// Load an override from a file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse an override from TOML string
    pub fn parse(content: &str) -> Result<Self> {
        let override_config: Self = toml::from_str(content)?;
        Ok(override_config)
    }

    /// Check if this override has any constraints
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty() && self.runtimes.is_empty()
    }

    /// Get constraints for a specific runtime
    pub fn get_constraints_for_runtime(&self, runtime_name: &str) -> &[ConstraintRule] {
        // First check runtime-specific overrides
        for runtime_override in &self.runtimes {
            if runtime_override.name == runtime_name {
                return &runtime_override.constraints;
            }
        }
        // Fall back to default constraints (for the main runtime)
        &self.constraints
    }
}

/// Apply overrides to a provider manifest
///
/// This function merges override constraints into the manifest.
/// Override constraints replace existing constraints for the same `when` pattern.
pub fn apply_override(manifest: &mut ProviderManifest, override_config: &ProviderOverride) {
    // Apply default constraints to the main runtime (same name as provider)
    if !override_config.constraints.is_empty() {
        if let Some(runtime) = manifest
            .runtimes
            .iter_mut()
            .find(|r| r.name == manifest.provider.name)
        {
            merge_constraints(&mut runtime.constraints, &override_config.constraints);
        }
    }

    // Apply runtime-specific overrides
    for runtime_override in &override_config.runtimes {
        if let Some(runtime) = manifest
            .runtimes
            .iter_mut()
            .find(|r| r.name == runtime_override.name)
        {
            merge_constraints(&mut runtime.constraints, &runtime_override.constraints);
        }
    }
}

/// Merge override constraints into existing constraints
///
/// Override constraints with the same `when` pattern replace existing ones.
/// New `when` patterns are appended.
fn merge_constraints(existing: &mut Vec<ConstraintRule>, overrides: &[ConstraintRule]) {
    for override_rule in overrides {
        // Find and replace existing rule with same `when` pattern
        if let Some(pos) = existing.iter().position(|r| r.when == override_rule.when) {
            existing[pos] = override_rule.clone();
        } else {
            // Append new rule
            existing.push(override_rule.clone());
        }
    }
}

/// Extract provider name from override filename
///
/// Examples:
/// - `yarn.override.toml` -> `Some("yarn")`
/// - `pre-commit.override.toml` -> `Some("pre-commit")`
/// - `invalid.toml` -> `None`
pub fn extract_provider_name(filename: &str) -> Option<&str> {
    filename.strip_suffix(".override.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_override() {
        let toml = r#"
[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=14, <21" }
]

[[constraints]]
when = "*"
requires = [
    { runtime = "git", version = ">=2.0", optional = true }
]
"#;
        let override_config = ProviderOverride::parse(toml).unwrap();
        assert_eq!(override_config.constraints.len(), 2);
        assert_eq!(override_config.constraints[0].when, "^1");
        assert_eq!(override_config.constraints[1].when, "*");
    }

    #[test]
    fn test_parse_runtime_specific_override() {
        let toml = r#"
[[runtimes]]
name = "npm"

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=16" }
]
"#;
        let override_config = ProviderOverride::parse(toml).unwrap();
        assert_eq!(override_config.runtimes.len(), 1);
        assert_eq!(override_config.runtimes[0].name, "npm");
        assert_eq!(override_config.runtimes[0].constraints.len(), 1);
    }

    #[test]
    fn test_apply_override() {
        let manifest_toml = r#"
[provider]
name = "yarn"

[[runtimes]]
name = "yarn"
executable = "yarn"

[[runtimes.constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=12, <23" }
]
"#;
        let mut manifest = ProviderManifest::parse(manifest_toml).unwrap();

        let override_toml = r#"
[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=14, <21" }
]
"#;
        let override_config = ProviderOverride::parse(override_toml).unwrap();

        apply_override(&mut manifest, &override_config);

        let runtime = &manifest.runtimes[0];
        assert_eq!(runtime.constraints.len(), 1);
        assert_eq!(runtime.constraints[0].requires[0].version, ">=14, <21");
    }

    #[test]
    fn test_apply_override_adds_new() {
        let manifest_toml = r#"
[provider]
name = "yarn"

[[runtimes]]
name = "yarn"
executable = "yarn"

[[runtimes.constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=12, <23" }
]
"#;
        let mut manifest = ProviderManifest::parse(manifest_toml).unwrap();

        let override_toml = r#"
[[constraints]]
when = ">=4"
requires = [
    { runtime = "node", version = ">=20" }
]
"#;
        let override_config = ProviderOverride::parse(override_toml).unwrap();

        apply_override(&mut manifest, &override_config);

        let runtime = &manifest.runtimes[0];
        assert_eq!(runtime.constraints.len(), 2);
    }

    #[test]
    fn test_extract_provider_name() {
        assert_eq!(extract_provider_name("yarn.override.toml"), Some("yarn"));
        assert_eq!(
            extract_provider_name("pre-commit.override.toml"),
            Some("pre-commit")
        );
        assert_eq!(extract_provider_name("invalid.toml"), None);
        assert_eq!(extract_provider_name("yarn.toml"), None);
    }
}
