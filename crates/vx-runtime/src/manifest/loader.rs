//! Manifest loading and storage
//!
//! This module wraps `vx_manifest::ManifestLoader` and provides a clean API
//! for loading manifests from directories, embedded sources, and in-memory objects.

use std::path::Path;
use tracing::trace;
use vx_manifest::{ManifestLoader, ProviderManifest};

/// Store of loaded provider manifests
///
/// Thin wrapper around `vx_manifest::ManifestLoader` that provides
/// the loading half of the old `ManifestRegistry`.
#[derive(Default)]
pub struct ManifestStore {
    loader: ManifestLoader,
}

impl ManifestStore {
    /// Create a new empty manifest store
    pub fn new() -> Self {
        Self::default()
    }

    /// Load manifests from a directory containing `<name>/provider.toml` files
    pub fn load_from_directory(&mut self, dir: &Path) -> anyhow::Result<usize> {
        let count = self.loader.load_from_dir(dir)?;
        trace!("loaded {} manifests from {:?}", count, dir);
        Ok(count)
    }

    /// Load manifests from a list of `ProviderManifest` objects
    pub fn load_from_manifests(&mut self, manifests: Vec<ProviderManifest>) {
        for manifest in manifests {
            self.loader.insert(manifest);
        }
    }

    /// Get a manifest by provider name
    pub fn get(&self, name: &str) -> Option<&ProviderManifest> {
        self.loader.get(name)
    }

    /// List all loaded manifest provider names
    pub fn names(&self) -> Vec<String> {
        self.loader.all().map(|m| m.provider.name.clone()).collect()
    }

    /// Iterate over all loaded manifests
    pub fn iter(&self) -> impl Iterator<Item = &ProviderManifest> {
        self.loader.all()
    }

    /// Find a runtime across all loaded manifests
    ///
    /// Returns the manifest and runtime definition if found (also searches aliases).
    pub fn find_runtime(
        &self,
        name: &str,
    ) -> Option<(&ProviderManifest, &vx_manifest::RuntimeDef)> {
        self.loader.find_runtime(name)
    }

    /// Get the number of loaded manifests
    pub fn len(&self) -> usize {
        self.loader.all().count()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
