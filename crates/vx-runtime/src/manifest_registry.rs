//! Manifest-driven provider registry
//!
//! This module provides an alternative registration mechanism that loads
//! provider metadata from `provider.toml` manifest files instead of
//! hard-coded static registration.
//!
//! ## Benefits
//!
//! - **Single source of truth**: Provider metadata is defined once in TOML
//! - **Declarative**: Easy to understand and modify without code changes
//! - **Extensible**: External providers can be loaded from disk
//!
//! ## Usage
//!
//! ```rust,ignore
//! use vx_runtime::ManifestRegistry;
//!
//! // Create registry and register factories
//! let mut registry = ManifestRegistry::new();
//! registry.register_factory("node", || Arc::new(node_provider()));
//!
//! // Load manifests
//! registry.load_from_directory("providers/")?;
//!
//! // Build the provider registry
//! let provider_registry = registry.build_registry();
//! ```

use crate::{Provider, ProviderRegistry};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};
use vx_manifest::{ManifestLoader, PlatformConstraint, ProviderManifest};

/// Manifest-driven provider registry
///
/// This registry can load providers from manifest files and integrate
/// with the existing static registration system.
#[derive(Default)]
pub struct ManifestRegistry {
    /// Loaded manifests
    loader: ManifestLoader,
    /// Provider factories by name
    factories: HashMap<String, Box<dyn Fn() -> Arc<dyn Provider> + Send + Sync>>,
}

impl ManifestRegistry {
    /// Create a new empty manifest registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a provider factory
    ///
    /// This associates a manifest name with a factory function that creates
    /// the actual Provider implementation.
    pub fn register_factory<F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> Arc<dyn Provider> + Send + Sync + 'static,
    {
        self.factories.insert(name.to_string(), Box::new(factory));
    }

    /// Load manifests from a directory
    pub fn load_from_directory(&mut self, dir: &Path) -> anyhow::Result<usize> {
        let count = self.loader.load_from_dir(dir)?;
        debug!("Loaded {} manifests from {:?}", count, dir);
        Ok(count)
    }

    /// Load manifests from a list of ProviderManifest objects
    pub fn load_from_manifests(&mut self, manifests: Vec<ProviderManifest>) {
        for manifest in manifests {
            self.loader.insert(manifest);
        }
    }

    /// Get a manifest by provider name
    pub fn get_manifest(&self, name: &str) -> Option<&ProviderManifest> {
        self.loader.get(name)
    }

    /// List all loaded manifest names
    pub fn manifest_names(&self) -> Vec<String> {
        self.loader.all().map(|m| m.provider.name.clone()).collect()
    }

    /// List all registered factory names
    pub fn factory_names(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    /// Build a ProviderRegistry from loaded manifests and factories
    ///
    /// This creates providers for all manifests that have registered factories.
    /// Manifests without factories are logged as warnings.
    pub fn build_registry(&self) -> ProviderRegistry {
        let registry = ProviderRegistry::new();

        for manifest in self.loader.all() {
            let name = &manifest.provider.name;

            if let Some(factory) = self.factories.get(name) {
                let provider = factory();
                registry.register(provider);
                debug!("Registered provider '{}' from manifest", name);
            } else {
                warn!(
                    "No factory registered for manifest '{}' - provider will not be available",
                    name
                );
            }
        }

        info!(
            "Built registry with {} providers from manifests",
            registry.providers().len()
        );

        registry
    }

    /// Build a ProviderRegistry using only registered factories (no manifest required)
    ///
    /// This is useful for backward compatibility when manifests are not available.
    pub fn build_registry_from_factories(&self) -> ProviderRegistry {
        let registry = ProviderRegistry::new();

        for (name, factory) in &self.factories {
            let provider = factory();
            registry.register(provider);
            debug!("Registered provider '{}' from factory", name);
        }

        info!(
            "Built registry with {} providers from factories",
            registry.providers().len()
        );

        registry
    }

    /// Check if a runtime is defined in any loaded manifest
    pub fn has_runtime(&self, name: &str) -> bool {
        self.loader.find_runtime(name).is_some()
    }

    /// Get runtime metadata from manifest
    pub fn get_runtime_metadata(&self, name: &str) -> Option<RuntimeMetadata> {
        let (manifest, runtime) = self.loader.find_runtime(name)?;

        // Combine provider-level and runtime-level platform constraints
        let platform_constraint = runtime
            .platform_constraint
            .clone()
            .or_else(|| manifest.provider.platform_constraint.clone());

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

        for manifest in self.loader.all() {
            // Skip provider if not supported on current platform
            if !manifest.is_current_platform_supported() {
                continue;
            }

            for runtime in manifest.supported_runtimes() {
                let platform_constraint = runtime
                    .platform_constraint
                    .clone()
                    .or_else(|| manifest.provider.platform_constraint.clone());

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

        for manifest in self.loader.all() {
            for runtime in &manifest.runtimes {
                let platform_constraint = runtime
                    .platform_constraint
                    .clone()
                    .or_else(|| manifest.provider.platform_constraint.clone());

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
}

/// Runtime metadata extracted from manifest
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
    pub ecosystem: Option<vx_manifest::Ecosystem>,
    /// Platform constraint (from runtime or provider level)
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
}
