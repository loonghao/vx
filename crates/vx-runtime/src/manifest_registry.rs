//! Manifest-driven provider registry
//!
//! This module provides an alternative registration mechanism that loads
//! provider metadata from `provider.toml` manifest files instead of
//! hard-coded static registration.
//!
//! ## Architecture (RFC 0029)
//!
//! `ManifestRegistry` composes three sub-modules:
//! - [`ManifestStore`](crate::manifest::loader::ManifestStore) — manifest loading
//! - [`ManifestIndex`](crate::manifest::index::ManifestIndex) — metadata indexing
//! - [`ProviderBuilder`](crate::manifest::builder::ProviderBuilder) — provider construction
//!
//! ## Usage
//!
//! ```rust,ignore
//! use vx_runtime::ManifestRegistry;
//!
//! let mut registry = ManifestRegistry::new();
//! registry.register_factory("node", || Arc::new(node_provider()));
//! registry.load_from_directory("providers/")?;
//!
//! // Preferred: get structured build result
//! let result = registry.build_registry_with_result();
//! if !result.errors.is_empty() {
//!     eprintln!("Build errors: {:?}", result.errors);
//! }
//! let provider_registry = result.registry;
//! ```

use crate::manifest::builder::{BuildResult, ProviderBuilder};
use crate::manifest::index::ManifestIndex;
use crate::manifest::loader::ManifestStore;
use crate::ProviderRegistry;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};
use vx_manifest::{PlatformConstraint, ProviderManifest};

// Re-export the new RuntimeMetadata from manifest::index
pub use crate::manifest::index::RuntimeMetadata;

/// Manifest-driven provider registry
///
/// Composes `ManifestStore` (loading), `ManifestIndex` (querying),
/// and `ProviderBuilder` (construction) into a single facade.
pub struct ManifestRegistry {
    /// Manifest storage
    store: ManifestStore,
    /// Provider builder with registered factories
    builder: ProviderBuilder,
}

impl Default for ManifestRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ManifestRegistry {
    /// Create a new empty manifest registry
    pub fn new() -> Self {
        Self {
            store: ManifestStore::new(),
            builder: ProviderBuilder::new(),
        }
    }

    // ======== Loading (delegated to ManifestStore) ========

    /// Register a provider factory
    ///
    /// This associates a manifest name with a factory function that creates
    /// the actual Provider implementation.
    pub fn register_factory<F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> Arc<dyn crate::Provider> + Send + Sync + 'static,
    {
        self.builder.register_factory(name, factory);
    }

    /// Load manifests from a directory
    pub fn load_from_directory(&mut self, dir: &Path) -> anyhow::Result<usize> {
        self.store.load_from_directory(dir)
    }

    /// Load manifests from a list of ProviderManifest objects
    pub fn load_from_manifests(&mut self, manifests: Vec<ProviderManifest>) {
        self.store.load_from_manifests(manifests);
    }

    /// Get a manifest by provider name
    pub fn get_manifest(&self, name: &str) -> Option<&ProviderManifest> {
        self.store.get(name)
    }

    /// List all loaded manifest names
    pub fn manifest_names(&self) -> Vec<String> {
        self.store.names()
    }

    /// List all registered factory names
    pub fn factory_names(&self) -> Vec<String> {
        self.builder.factory_names()
    }

    // ======== Building (delegated to ProviderBuilder) ========

    /// Build a ProviderRegistry from loaded manifests and factories (structured result)
    ///
    /// This is the **preferred** method. Returns a `BuildResult` with errors and
    /// warnings instead of silently logging them.
    pub fn build_registry_with_result(&self) -> BuildResult {
        let manifests: Vec<_> = self.store.iter().cloned().collect();
        self.builder.build(&manifests)
    }

    /// Build a ProviderRegistry from loaded manifests and factories
    ///
    /// **Backward-compatible**: logs warnings for missing factories.
    /// Prefer `build_registry_with_result()` for structured error handling.
    pub fn build_registry(&self) -> ProviderRegistry {
        let result = self.build_registry_with_result();

        // Log errors as warnings for backward compatibility
        for error in &result.errors {
            warn!(
                "No factory registered for manifest '{}' - provider will not be available",
                error.provider
            );
        }

        info!(
            "loaded {} providers from manifests",
            result.registry.providers().len()
        );

        result.registry
    }

    /// Build a ProviderRegistry using only registered factories (no manifest required)
    ///
    /// This is useful for backward compatibility when manifests are not available.
    pub fn build_registry_from_factories(&self) -> ProviderRegistry {
        self.builder.build_from_factories()
    }

    // ======== Querying (delegated to ManifestIndex) ========

    /// Build a `ManifestIndex` from the currently loaded manifests
    ///
    /// The index provides O(1) lookups by runtime name or alias
    /// and uses `PlatformConstraint::intersect()` for constraint merging.
    pub fn build_index(&self) -> ManifestIndex {
        ManifestIndex::from_manifest_iter(self.store.iter())
    }

    /// Check if a runtime is defined in any loaded manifest
    pub fn has_runtime(&self, name: &str) -> bool {
        self.store.find_runtime(name).is_some()
    }

    /// Get runtime metadata from manifest
    ///
    /// Uses `PlatformConstraint::intersect()` to merge provider + runtime constraints.
    pub fn get_runtime_metadata(&self, name: &str) -> Option<RuntimeMetadata> {
        let (manifest, runtime) = self.store.find_runtime(name)?;

        // Merge provider-level and runtime-level platform constraints via intersection
        let platform_constraint = merge_platform_constraints(
            &manifest.provider.platform_constraint,
            &runtime.platform_constraint,
        );

        Some(RuntimeMetadata {
            name: runtime.name.clone(),
            description: runtime.description.clone(),
            executable: runtime.executable.clone(),
            aliases: runtime.aliases.clone(),
            provider_name: manifest.provider.name.clone(),
            ecosystem: manifest.provider.ecosystem,
            platform_constraint,
        })
    }

    /// Get all runtime metadata for runtimes supported on the current platform
    pub fn get_supported_runtimes(&self) -> Vec<RuntimeMetadata> {
        let mut result = Vec::new();

        for manifest in self.store.iter() {
            // Skip provider if not supported on current platform
            if !manifest.is_current_platform_supported() {
                continue;
            }

            for runtime in manifest.supported_runtimes() {
                let platform_constraint = merge_platform_constraints(
                    &manifest.provider.platform_constraint,
                    &runtime.platform_constraint,
                );

                result.push(RuntimeMetadata {
                    name: runtime.name.clone(),
                    description: runtime.description.clone(),
                    executable: runtime.executable.clone(),
                    aliases: runtime.aliases.clone(),
                    provider_name: manifest.provider.name.clone(),
                    ecosystem: manifest.provider.ecosystem,
                    platform_constraint,
                });
            }
        }

        result
    }

    /// Get all runtime metadata (including unsupported platforms)
    pub fn get_all_runtimes(&self) -> Vec<RuntimeMetadata> {
        let mut result = Vec::new();

        for manifest in self.store.iter() {
            for runtime in &manifest.runtimes {
                let platform_constraint = merge_platform_constraints(
                    &manifest.provider.platform_constraint,
                    &runtime.platform_constraint,
                );

                result.push(RuntimeMetadata {
                    name: runtime.name.clone(),
                    description: runtime.description.clone(),
                    executable: runtime.executable.clone(),
                    aliases: runtime.aliases.clone(),
                    provider_name: manifest.provider.name.clone(),
                    ecosystem: manifest.provider.ecosystem,
                    platform_constraint,
                });
            }
        }

        result
    }

    // ======== Direct access to sub-components ========

    /// Get a reference to the underlying manifest store
    pub fn store(&self) -> &ManifestStore {
        &self.store
    }

    /// Get a reference to the underlying provider builder
    pub fn provider_builder(&self) -> &ProviderBuilder {
        &self.builder
    }
}

/// Merge provider-level and runtime-level platform constraints via intersection
///
/// - Both present: intersect (most restrictive)
/// - One present: use that one
/// - Neither present: None (all platforms)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_manifest(dir: &Path, name: &str) {
        let provider_dir = dir.join(name);
        fs::create_dir_all(&provider_dir).unwrap();

        let manifest = format!(
            r#"
[provider]
name = "{name}"

[[runtimes]]
name = "{name}"
executable = "{name}"
"#
        );

        fs::write(provider_dir.join("provider.toml"), manifest).unwrap();
    }

    #[test]
    fn test_manifest_registry_load() {
        let temp_dir = TempDir::new().unwrap();
        create_test_manifest(temp_dir.path(), "test-provider");

        let mut registry = ManifestRegistry::new();
        let count = registry.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(count, 1);
        assert!(registry.get_manifest("test-provider").is_some());
    }

    #[test]
    fn test_runtime_metadata() {
        let temp_dir = TempDir::new().unwrap();

        let manifest = r#"
[provider]
name = "test"
ecosystem = "nodejs"

[[runtimes]]
name = "test-runtime"
description = "A test runtime"
executable = "test-bin"
aliases = ["tr", "test"]
"#;

        let provider_dir = temp_dir.path().join("test");
        fs::create_dir_all(&provider_dir).unwrap();
        fs::write(provider_dir.join("provider.toml"), manifest).unwrap();

        let mut registry = ManifestRegistry::new();
        registry.load_from_directory(temp_dir.path()).unwrap();

        let metadata = registry.get_runtime_metadata("test-runtime").unwrap();
        assert_eq!(metadata.name, "test-runtime");
        assert_eq!(metadata.executable, "test-bin");
        assert_eq!(metadata.aliases, vec!["tr", "test"]);

        // Should also find by alias
        let metadata_by_alias = registry.get_runtime_metadata("tr").unwrap();
        assert_eq!(metadata_by_alias.name, "test-runtime");
    }

    #[test]
    fn test_build_registry_with_result() {
        let mut registry = ManifestRegistry::new();

        // Load a manifest without a factory
        let manifest = ProviderManifest {
            provider: vx_manifest::ProviderMeta {
                name: "no-factory".to_string(),
                description: Some("no factory".to_string()),
                homepage: None,
                repository: None,
                ecosystem: None,
                platform_constraint: None,
            },
            runtimes: vec![],
        };
        registry.load_from_manifests(vec![manifest]);

        let result = registry.build_registry_with_result();
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].provider, "no-factory");
    }

    #[test]
    fn test_build_index() {
        let temp_dir = TempDir::new().unwrap();

        let manifest_toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "myrt"
executable = "myrt"
aliases = ["mr"]
"#;

        let provider_dir = temp_dir.path().join("test");
        fs::create_dir_all(&provider_dir).unwrap();
        fs::write(provider_dir.join("provider.toml"), manifest_toml).unwrap();

        let mut registry = ManifestRegistry::new();
        registry.load_from_directory(temp_dir.path()).unwrap();

        let index = registry.build_index();
        assert!(index.has_runtime("myrt"));
        assert!(index.has_runtime("mr"));
        assert!(!index.has_runtime("nonexistent"));
    }
}
