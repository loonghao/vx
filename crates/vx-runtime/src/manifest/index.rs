//! Runtime metadata index
//!
//! Pre-built index for fast runtime/alias/provider lookups.
//! Uses `PlatformConstraint::intersect()` for merging provider + runtime constraints.

use std::collections::HashMap;
use vx_manifest::{Ecosystem, PlatformConstraint, ProviderManifest};

/// Runtime metadata extracted from manifests
#[derive(Debug, Clone)]
pub struct RuntimeMetadata {
    /// Runtime name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Executable name
    pub executable: String,
    /// Aliases
    pub aliases: Vec<String>,
    /// Provider name
    pub provider_name: String,
    /// Ecosystem
    pub ecosystem: Option<Ecosystem>,
    /// Platform constraint (merged from provider + runtime level via intersection)
    pub platform_constraint: Option<PlatformConstraint>,
}

impl RuntimeMetadata {
    /// Check if this runtime is supported on the current platform
    pub fn is_current_platform_supported(&self) -> bool {
        self.platform_constraint
            .as_ref()
            .is_none_or(|c| c.is_current_platform_supported())
    }

    /// Get a human-readable platform description
    pub fn platform_description(&self) -> Option<String> {
        self.platform_constraint
            .as_ref()
            .and_then(|c| c.description())
    }

    /// Get a short platform label for display
    pub fn platform_label(&self) -> Option<String> {
        self.platform_constraint
            .as_ref()
            .and_then(|c| c.short_label())
    }
}

/// Provider-level metadata
#[derive(Debug, Clone)]
pub struct ProviderMetadata {
    /// Provider name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Ecosystem
    pub ecosystem: Option<Ecosystem>,
    /// Provider-level platform constraint
    pub platform_constraint: Option<PlatformConstraint>,
}

/// Pre-built index for fast runtime and provider lookups
///
/// Built from a set of `ProviderManifest` objects, this index provides
/// O(1) lookups by runtime name or alias.
pub struct ManifestIndex {
    /// Runtime name → RuntimeMetadata
    runtimes: HashMap<String, RuntimeMetadata>,
    /// Alias → canonical runtime name
    aliases: HashMap<String, String>,
    /// Provider name → ProviderMetadata
    providers: HashMap<String, ProviderMetadata>,
}

impl ManifestIndex {
    /// Build an index from a slice of manifests
    pub fn from_manifests(manifests: &[ProviderManifest]) -> Self {
        let mut runtimes = HashMap::new();
        let mut aliases = HashMap::new();
        let mut providers = HashMap::new();

        for manifest in manifests {
            let provider_name = &manifest.provider.name;

            providers.insert(
                provider_name.clone(),
                ProviderMetadata {
                    name: provider_name.clone(),
                    description: manifest.provider.description.clone(),
                    ecosystem: manifest.provider.ecosystem,
                    platform_constraint: manifest.provider.platform_constraint.clone(),
                },
            );

            for runtime in &manifest.runtimes {
                // Merge provider + runtime platform constraints via intersection
                let platform_constraint = merge_platform_constraints(
                    &manifest.provider.platform_constraint,
                    &runtime.platform_constraint,
                );

                let metadata = RuntimeMetadata {
                    name: runtime.name.clone(),
                    description: runtime.description.clone(),
                    executable: runtime.executable.clone(),
                    aliases: runtime.aliases.clone(),
                    provider_name: provider_name.clone(),
                    ecosystem: manifest.provider.ecosystem,
                    platform_constraint,
                };

                // Register canonical name
                runtimes.insert(runtime.name.clone(), metadata);

                // Register aliases
                for alias in &runtime.aliases {
                    aliases.insert(alias.clone(), runtime.name.clone());
                }
            }
        }

        Self {
            runtimes,
            aliases,
            providers,
        }
    }

    /// Build an index from an iterator of manifest references
    pub fn from_manifest_iter<'a>(manifests: impl Iterator<Item = &'a ProviderManifest>) -> Self {
        let collected: Vec<_> = manifests.cloned().collect();
        Self::from_manifests(&collected)
    }

    /// Resolve an alias to its canonical runtime name
    pub fn resolve_alias<'a>(&'a self, name: &'a str) -> &'a str {
        self.aliases.get(name).map(|s| s.as_str()).unwrap_or(name)
    }

    /// Get runtime metadata by name or alias
    pub fn get_runtime(&self, name: &str) -> Option<&RuntimeMetadata> {
        let canonical = self.resolve_alias(name);
        self.runtimes.get(canonical)
    }

    /// Check if a runtime exists (by name or alias)
    pub fn has_runtime(&self, name: &str) -> bool {
        self.get_runtime(name).is_some()
    }

    /// Get provider metadata by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderMetadata> {
        self.providers.get(name)
    }

    /// Get the effective platform constraint for a runtime
    ///
    /// This returns the already-merged (intersected) constraint stored in RuntimeMetadata.
    pub fn get_platform_constraint(&self, runtime: &str) -> Option<&PlatformConstraint> {
        self.get_runtime(runtime)
            .and_then(|r| r.platform_constraint.as_ref())
    }

    /// Get all runtimes supported on the current platform
    pub fn get_supported_runtimes(&self) -> Vec<&RuntimeMetadata> {
        self.runtimes
            .values()
            .filter(|r| r.is_current_platform_supported())
            .collect()
    }

    /// Get all runtimes (including unsupported platforms)
    pub fn get_all_runtimes(&self) -> Vec<&RuntimeMetadata> {
        self.runtimes.values().collect()
    }

    /// Get the number of indexed runtimes
    pub fn runtime_count(&self) -> usize {
        self.runtimes.len()
    }

    /// Get the number of indexed providers
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

/// Merge provider-level and runtime-level platform constraints via intersection
///
/// If both levels define constraints, the result is the intersection.
/// If only one level defines constraints, that constraint is used.
/// If neither defines constraints, returns None (all platforms supported).
fn merge_platform_constraints(
    provider: &Option<PlatformConstraint>,
    runtime: &Option<PlatformConstraint>,
) -> Option<PlatformConstraint> {
    match (provider, runtime) {
        (Some(p), Some(r)) => {
            let merged = p.intersect(r);
            if merged.is_empty() {
                None
            } else {
                Some(merged)
            }
        }
        (Some(p), None) => {
            if p.is_empty() {
                None
            } else {
                Some(p.clone())
            }
        }
        (None, Some(r)) => {
            if r.is_empty() {
                None
            } else {
                Some(r.clone())
            }
        }
        (None, None) => None,
    }
}
