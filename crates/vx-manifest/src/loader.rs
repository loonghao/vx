//! Manifest loader for discovering and loading provider.toml files

use crate::{
    apply_override, extract_provider_name, ManifestError, ProviderManifest, ProviderOverride,
    Result,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Manifest loader - discovers and loads provider.toml files
#[derive(Debug, Default)]
pub struct ManifestLoader {
    /// Loaded manifests by provider name
    manifests: HashMap<String, ProviderManifest>,
    /// Manifest file paths by provider name
    paths: HashMap<String, PathBuf>,
    /// Pending overrides by provider name (applied when building)
    overrides: HashMap<String, Vec<ProviderOverride>>,
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

        let entries = std::fs::read_dir(providers_dir).map_err(ManifestError::Io)?;

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

    /// Load manifests from embedded (name, content) tuples.
    /// Later entries with the same provider name override earlier ones.
    pub fn load_embedded<'a, I>(&mut self, manifests: I) -> Result<usize>
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut count = 0;
        for (name, content) in manifests {
            match ProviderManifest::parse(content) {
                Ok(manifest) => {
                    let provider_name = manifest.provider.name.clone();
                    if provider_name != name {
                        tracing::warn!(
                            "Manifest name mismatch: embedded key '{}' differs from provider '{}'; using provider name",
                            name,
                            provider_name
                        );
                    }
                    self.insert(manifest);
                    count += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to parse embedded manifest '{}': {}", name, e);
                }
            }
        }
        Ok(count)
    }

    /// Insert a manifest directly (used for overlays/overrides).
    pub fn insert(&mut self, manifest: ProviderManifest) {
        let name = manifest.provider.name.clone();
        self.manifests.insert(name.clone(), manifest);
        // Unknown path for embedded/override entries; use empty PathBuf as placeholder.
        self.paths.entry(name).or_insert_with(PathBuf::new);
    }

    /// Load override files from a directory
    ///
    /// Override files are named `<provider>.override.toml` and contain
    /// constraint overrides for the corresponding provider.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// loader.load_overrides_from_dir(Path::new("~/.vx/providers"))?;
    /// ```
    pub fn load_overrides_from_dir(&mut self, dir: &Path) -> Result<usize> {
        let mut count = 0;

        if !dir.exists() {
            return Ok(0);
        }

        let entries = std::fs::read_dir(dir).map_err(ManifestError::Io)?;

        for entry in entries {
            let entry = entry.map_err(ManifestError::Io)?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(provider_name) = extract_provider_name(filename) {
                        match ProviderOverride::load(&path) {
                            Ok(override_config) => {
                                if !override_config.is_empty() {
                                    self.overrides
                                        .entry(provider_name.to_string())
                                        .or_default()
                                        .push(override_config);
                                    count += 1;
                                    tracing::debug!(
                                        "Loaded override for '{}' from {:?}",
                                        provider_name,
                                        path
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load override from {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Apply all loaded overrides to manifests
    ///
    /// This should be called after loading all manifests and overrides.
    pub fn apply_overrides(&mut self) {
        for (provider_name, overrides) in &self.overrides {
            if let Some(manifest) = self.manifests.get_mut(provider_name) {
                for override_config in overrides {
                    apply_override(manifest, override_config);
                }
                tracing::debug!(
                    "Applied {} override(s) to provider '{}'",
                    overrides.len(),
                    provider_name
                );
            } else {
                tracing::warn!(
                    "Override for '{}' has no matching manifest - ignored",
                    provider_name
                );
            }
        }
    }

    /// Consume the loader and return all loaded manifests with overrides applied.
    pub fn into_manifests(mut self) -> Vec<ProviderManifest> {
        self.apply_overrides();
        self.manifests.into_values().collect()
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
    pub fn find_runtime(
        &self,
        runtime_name: &str,
    ) -> Option<(&ProviderManifest, &crate::RuntimeDef)> {
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

    fn create_test_manifest_with_constraints(dir: &Path, name: &str) {
        let provider_dir = dir.join(name);
        fs::create_dir_all(&provider_dir).unwrap();

        let manifest = format!(
            r#"
[provider]
name = "{name}"

[[runtimes]]
name = "{name}"
executable = "{name}"

[[runtimes.constraints]]
when = "^1"
requires = [
    {{ runtime = "node", version = ">=12, <23" }}
]
"#
        );

        fs::write(provider_dir.join("provider.toml"), manifest).unwrap();
    }

    fn create_override_file(dir: &Path, provider_name: &str, content: &str) {
        let filename = format!("{}.override.toml", provider_name);
        fs::write(dir.join(filename), content).unwrap();
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

    #[test]
    fn test_load_overrides_from_dir() {
        let temp_dir = TempDir::new().unwrap();

        // Create a provider manifest
        create_test_manifest_with_constraints(temp_dir.path(), "yarn");

        // Create an override file
        let override_content = r#"
[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=14, <21" }
]
"#;
        create_override_file(temp_dir.path(), "yarn", override_content);

        let mut loader = ManifestLoader::new();
        loader.load_from_dir(temp_dir.path()).unwrap();
        let override_count = loader.load_overrides_from_dir(temp_dir.path()).unwrap();

        assert_eq!(override_count, 1);

        // Get manifests with overrides applied
        let manifests = loader.into_manifests();
        assert_eq!(manifests.len(), 1);

        let manifest = &manifests[0];
        let runtime = manifest.get_runtime("yarn").unwrap();
        assert_eq!(runtime.constraints.len(), 1);
        assert_eq!(runtime.constraints[0].requires[0].version, ">=14, <21");
    }

    #[test]
    fn test_override_without_manifest() {
        let temp_dir = TempDir::new().unwrap();

        // Create an override file without a corresponding manifest
        let override_content = r#"
[[constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=18" }
]
"#;
        create_override_file(temp_dir.path(), "nonexistent", override_content);

        let mut loader = ManifestLoader::new();
        let override_count = loader.load_overrides_from_dir(temp_dir.path()).unwrap();

        // Override is loaded but won't be applied (no matching manifest)
        assert_eq!(override_count, 1);

        let manifests = loader.into_manifests();
        assert!(manifests.is_empty());
    }

    #[test]
    fn test_multiple_overrides() {
        let temp_dir = TempDir::new().unwrap();
        let user_dir = temp_dir.path().join("user");
        let project_dir = temp_dir.path().join("project");
        fs::create_dir_all(&user_dir).unwrap();
        fs::create_dir_all(&project_dir).unwrap();

        // Create a provider manifest in user dir
        create_test_manifest_with_constraints(&user_dir, "yarn");

        // Create user-level override
        let user_override = r#"
[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=14, <21" }
]
"#;
        create_override_file(&user_dir, "yarn", user_override);

        // Create project-level override (should take precedence)
        let project_override = r#"
[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=16, <20" }
]
"#;
        create_override_file(&project_dir, "yarn", project_override);

        let mut loader = ManifestLoader::new();
        loader.load_from_dir(&user_dir).unwrap();
        loader.load_overrides_from_dir(&user_dir).unwrap();
        loader.load_overrides_from_dir(&project_dir).unwrap();

        let manifests = loader.into_manifests();
        let manifest = &manifests[0];
        let runtime = manifest.get_runtime("yarn").unwrap();

        // Project override should win (applied last)
        assert_eq!(runtime.constraints[0].requires[0].version, ">=16, <20");
    }
}
