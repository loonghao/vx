//! Extension discovery - finds and loads extensions from various sources

use crate::{Extension, ExtensionConfig, ExtensionSource};
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};
use vx_paths::VxPaths;

/// Extension discovery service
pub struct ExtensionDiscovery {
    /// User extensions directory (~/.vx/extensions/)
    user_dir: PathBuf,
    /// Dev extensions directory (~/.vx/extensions-dev/)
    dev_dir: PathBuf,
    /// Project extensions directory (.vx/extensions/)
    project_dir: Option<PathBuf>,
}

impl ExtensionDiscovery {
    /// Create a new extension discovery service
    pub fn new() -> anyhow::Result<Self> {
        let vx_paths = VxPaths::new()?;
        let base_dir = &vx_paths.base_dir;

        Ok(Self {
            user_dir: base_dir.join("extensions"),
            dev_dir: base_dir.join("extensions-dev"),
            project_dir: None,
        })
    }

    /// Set the project directory for project-level extension discovery
    pub fn with_project_dir(mut self, project_dir: PathBuf) -> Self {
        self.project_dir = Some(project_dir.join(".vx").join("extensions"));
        self
    }

    /// Discover all extensions from all sources
    pub async fn discover_all(&self) -> anyhow::Result<Vec<Extension>> {
        let mut extensions = Vec::new();

        // 1. Dev extensions (highest priority)
        if self.dev_dir.exists() {
            debug!("Scanning dev extensions: {:?}", self.dev_dir);
            extensions.extend(
                self.scan_directory(&self.dev_dir, ExtensionSource::Dev)
                    .await?,
            );
        }

        // 2. Project extensions
        if let Some(ref project_dir) = self.project_dir {
            if project_dir.exists() {
                debug!("Scanning project extensions: {:?}", project_dir);
                extensions.extend(
                    self.scan_directory(project_dir, ExtensionSource::Project)
                        .await?,
                );
            }
        }

        // 3. User extensions
        if self.user_dir.exists() {
            debug!("Scanning user extensions: {:?}", self.user_dir);
            extensions.extend(
                self.scan_directory(&self.user_dir, ExtensionSource::User)
                    .await?,
            );
        }

        // Sort by priority (higher priority first)
        extensions.sort_by(|a, b| b.source.priority().cmp(&a.source.priority()));

        Ok(extensions)
    }

    /// Find a specific extension by name
    pub async fn find_extension(&self, name: &str) -> anyhow::Result<Option<Extension>> {
        // Search in priority order: dev -> project -> user -> builtin

        // 1. Dev extensions
        if let Some(ext) = self
            .find_in_directory(&self.dev_dir, name, ExtensionSource::Dev)
            .await?
        {
            return Ok(Some(ext));
        }

        // 2. Project extensions
        if let Some(ref project_dir) = self.project_dir {
            if let Some(ext) = self
                .find_in_directory(project_dir, name, ExtensionSource::Project)
                .await?
            {
                return Ok(Some(ext));
            }
        }

        // 3. User extensions
        if let Some(ext) = self
            .find_in_directory(&self.user_dir, name, ExtensionSource::User)
            .await?
        {
            return Ok(Some(ext));
        }

        Ok(None)
    }

    /// Scan a directory for extensions
    async fn scan_directory(
        &self,
        dir: &Path,
        source: ExtensionSource,
    ) -> anyhow::Result<Vec<Extension>> {
        let mut extensions = Vec::new();

        if !dir.exists() {
            return Ok(extensions);
        }

        let entries = std::fs::read_dir(dir)?;

        for entry in entries.flatten() {
            let path = entry.path();

            // Follow symlinks for dev extensions
            let path = if path.is_symlink() {
                std::fs::read_link(&path).unwrap_or(path)
            } else {
                path
            };

            if path.is_dir() {
                if let Some(ext) = self.load_extension(&path, source).await? {
                    extensions.push(ext);
                }
            }
        }

        Ok(extensions)
    }

    /// Find an extension in a specific directory
    async fn find_in_directory(
        &self,
        dir: &Path,
        name: &str,
        source: ExtensionSource,
    ) -> anyhow::Result<Option<Extension>> {
        let ext_path = dir.join(name);

        // Follow symlinks
        let ext_path = if ext_path.is_symlink() {
            std::fs::read_link(&ext_path).unwrap_or(ext_path)
        } else {
            ext_path
        };

        if ext_path.is_dir() {
            return self.load_extension(&ext_path, source).await;
        }

        Ok(None)
    }

    /// Load an extension from a directory
    async fn load_extension(
        &self,
        path: &Path,
        source: ExtensionSource,
    ) -> anyhow::Result<Option<Extension>> {
        let config_path = path.join("vx-extension.toml");

        if !config_path.exists() {
            trace!("No vx-extension.toml found in {:?}", path);
            return Ok(None);
        }

        match ExtensionConfig::from_file(&config_path) {
            Ok(config) => {
                let name = config.extension.name.clone();
                debug!("Loaded extension '{}' from {:?}", name, path);

                Ok(Some(Extension {
                    name,
                    config,
                    path: path.to_path_buf(),
                    source,
                }))
            }
            Err(e) => {
                warn!("Failed to load extension from {:?}: {}", path, e);
                Ok(None)
            }
        }
    }

    /// Get the user extensions directory
    pub fn user_extensions_dir(&self) -> &Path {
        &self.user_dir
    }

    /// Get the dev extensions directory
    pub fn dev_extensions_dir(&self) -> &Path {
        &self.dev_dir
    }

    /// Get the project extensions directory
    pub fn project_extensions_dir(&self) -> Option<&Path> {
        self.project_dir.as_deref()
    }
}

impl Default for ExtensionDiscovery {
    fn default() -> Self {
        Self::new().expect("Failed to create extension discovery")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_discover_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let ext_dir = temp_dir.path().join("extensions");
        fs::create_dir_all(&ext_dir).unwrap();

        // Create a test extension
        let test_ext = ext_dir.join("test-ext");
        fs::create_dir_all(&test_ext).unwrap();
        fs::write(
            test_ext.join("vx-extension.toml"),
            r#"
[extension]
name = "test-ext"
version = "1.0.0"
"#,
        )
        .unwrap();

        let discovery = ExtensionDiscovery {
            user_dir: ext_dir,
            dev_dir: temp_dir.path().join("extensions-dev"),
            project_dir: None,
        };

        let extensions = discovery.discover_all().await.unwrap();
        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0].name, "test-ext");
    }

    #[tokio::test]
    async fn test_find_extension() {
        let temp_dir = TempDir::new().unwrap();
        let ext_dir = temp_dir.path().join("extensions");
        fs::create_dir_all(&ext_dir).unwrap();

        // Create a test extension
        let test_ext = ext_dir.join("my-ext");
        fs::create_dir_all(&test_ext).unwrap();
        fs::write(
            test_ext.join("vx-extension.toml"),
            r#"
[extension]
name = "my-ext"
"#,
        )
        .unwrap();

        let discovery = ExtensionDiscovery {
            user_dir: ext_dir,
            dev_dir: temp_dir.path().join("extensions-dev"),
            project_dir: None,
        };

        let ext = discovery.find_extension("my-ext").await.unwrap();
        assert!(ext.is_some());
        assert_eq!(ext.unwrap().name, "my-ext");

        let not_found = discovery.find_extension("nonexistent").await.unwrap();
        assert!(not_found.is_none());
    }
}
