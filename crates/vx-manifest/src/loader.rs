//! Manifest loader for discovering and loading provider.toml files

use crate::{ManifestError, ProviderManifest, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Manifest loader - discovers and loads provider.toml files
#[derive(Debug, Default)]
pub struct ManifestLoader {
    /// Loaded manifests by provider name
    manifests: HashMap<String, ProviderManifest>,
    /// Manifest file paths by provider name
    paths: HashMap<String, PathBuf>,
}

impl ManifestLoader {
    /// Create a new manifest loader
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all manifests from a providers directory
    pub fn load_from_dir(&mut self, providers_dir: &Path) -> Result<usize> {
        let mut count = 0;

        if !providers_dir.exists() {
            return Ok(0);
        }

        let entries = std::fs::read_dir(providers_dir)
            .map_err(ManifestError::Io)?;

        for entry in entries {
            let entry = entry.map_err(ManifestError::Io)?;
            let path = entry.path();

            if path.is_dir() {
                let manifest_path = path.join("provider.toml");
                if manifest_path.exists() {
                    match ProviderManifest::load(&manifest_path) {
                        Ok(manifest) => {
                            let name = manifest.provider.name.clone();
                            self.paths.insert(name.clone(), manifest_path);
                            self.manifests.insert(name, manifest);
                            count += 1;
                        }
                        Err(e) => {
                            // Log warning but continue loading other manifests
                            tracing::warn!(
                                "Failed to load manifest from {:?}: {}",
                                manifest_path,
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Load a single manifest file
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        let manifest = ProviderManifest::load(path)?;
        let name = manifest.provider.name.clone();
        self.paths.insert(name.clone(), path.to_path_buf());
        self.manifests.insert(name, manifest);
        Ok(())
    }

    /// Get a manifest by provider name
    pub fn get(&self, name: &str) -> Option<&ProviderManifest> {
        self.manifests.get(name)
    }

    /// Get all loaded manifests
    pub fn all(&self) -> impl Iterator<Item = &ProviderManifest> {
        self.manifests.values()
    }

    /// Get the number of loaded manifests
    pub fn len(&self) -> usize {
        self.manifests.len()
    }

    /// Check if no manifests are loaded
    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }

    /// Get manifest file path for a provider
    pub fn get_path(&self, name: &str) -> Option<&Path> {
        self.paths.get(name).map(|p| p.as_path())
    }

    /// Find a runtime definition across all manifests
    pub fn find_runtime(&self, runtime_name: &str) -> Option<(&ProviderManifest, &crate::RuntimeDef)> {
        for manifest in self.manifests.values() {
            if let Some(runtime) = manifest.get_runtime(runtime_name) {
                return Some((manifest, runtime));
            }
        }
        None
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
    fn test_load_from_dir() {
        let temp_dir = TempDir::new().unwrap();
        
        create_test_manifest(temp_dir.path(), "test1");
        create_test_manifest(temp_dir.path(), "test2");
        
        let mut loader = ManifestLoader::new();
        let count = loader.load_from_dir(temp_dir.path()).unwrap();
        
        assert_eq!(count, 2);
        assert_eq!(loader.len(), 2);
        assert!(loader.get("test1").is_some());
        assert!(loader.get("test2").is_some());
    }

    #[test]
    fn test_find_runtime() {
        let temp_dir = TempDir::new().unwrap();
        create_test_manifest(temp_dir.path(), "myruntime");
        
        let mut loader = ManifestLoader::new();
        loader.load_from_dir(temp_dir.path()).unwrap();
        
        let result = loader.find_runtime("myruntime");
        assert!(result.is_some());
        
        let (manifest, runtime) = result.unwrap();
        assert_eq!(manifest.provider.name, "myruntime");
        assert_eq!(runtime.name, "myruntime");
    }
}
