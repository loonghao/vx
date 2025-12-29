//! Configuration inheritance system
//!
//! Supports inheriting configuration from:
//! - Local files: `extends = "./base.toml"`
//! - Remote URLs: `extends = "https://example.com/base.toml"`
//! - GitHub: `extends = "github:user/repo/path/to/config.toml"`
//! - Installed extensions: `extends = "ext:base-extension"`

use crate::config::ExtensionConfig;
use crate::error::{ExtensionError, ExtensionResult};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Configuration inheritance resolver
pub struct ConfigInheritance {
    /// Maximum inheritance depth to prevent infinite loops
    max_depth: usize,
    /// Cache directory for remote configs
    cache_dir: PathBuf,
}

impl ConfigInheritance {
    /// Create a new inheritance resolver
    pub fn new() -> ExtensionResult<Self> {
        let vx_paths = vx_paths::VxPaths::new().map_err(|e| ExtensionError::Io {
            message: format!("Failed to get vx paths: {}", e),
            path: None,
            source: std::io::Error::other(e.to_string()),
        })?;

        Ok(Self {
            max_depth: 10,
            cache_dir: vx_paths.cache_dir,
        })
    }

    /// Set maximum inheritance depth
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Resolve and merge inherited configurations
    pub fn resolve(
        &self,
        config: ExtensionConfig,
        base_path: &Path,
    ) -> ExtensionResult<ExtensionConfig> {
        let mut seen = HashSet::new();
        self.resolve_recursive(config, base_path, &mut seen, 0)
    }

    /// Recursively resolve inheritance
    fn resolve_recursive(
        &self,
        config: ExtensionConfig,
        base_path: &Path,
        seen: &mut HashSet<String>,
        depth: usize,
    ) -> ExtensionResult<ExtensionConfig> {
        if depth >= self.max_depth {
            return Err(ExtensionError::CircularDependency {
                chain: format!(
                    "Maximum inheritance depth ({}) exceeded. Check for circular extends.",
                    self.max_depth
                ),
            });
        }

        let Some(extends) = &config.extends else {
            return Ok(config);
        };

        // Check for circular inheritance
        if seen.contains(extends) {
            return Err(ExtensionError::CircularDependency {
                chain: format!(
                    "Circular inheritance detected: {} -> {}",
                    seen.iter().cloned().collect::<Vec<_>>().join(" -> "),
                    extends
                ),
            });
        }

        seen.insert(extends.clone());

        // Load the parent config
        let (parent_config, parent_path) = self.load_parent(extends, base_path)?;

        // Recursively resolve parent's inheritance
        let resolved_parent =
            self.resolve_recursive(parent_config, &parent_path, seen, depth + 1)?;

        // Merge configs (child overrides parent)
        Ok(self.merge_configs(resolved_parent, config))
    }

    /// Load parent configuration from various sources
    fn load_parent(
        &self,
        extends: &str,
        base_path: &Path,
    ) -> ExtensionResult<(ExtensionConfig, PathBuf)> {
        // Local file
        if extends.starts_with("./") || extends.starts_with("../") || extends.ends_with(".toml") {
            let parent_path = base_path.parent().unwrap_or(base_path).join(extends);
            let config = ExtensionConfig::from_file(&parent_path)?;
            return Ok((config, parent_path));
        }

        // Extension reference: ext:name
        if let Some(ext_name) = extends.strip_prefix("ext:") {
            return self.load_from_extension(ext_name);
        }

        // GitHub reference: github:user/repo/path
        if let Some(github_ref) = extends.strip_prefix("github:") {
            return self.load_from_github(github_ref);
        }

        // HTTP(S) URL
        if extends.starts_with("http://") || extends.starts_with("https://") {
            return self.load_from_url(extends);
        }

        // Default: treat as local file
        let parent_path = base_path.parent().unwrap_or(base_path).join(extends);
        let config = ExtensionConfig::from_file(&parent_path)?;
        Ok((config, parent_path))
    }

    /// Load config from an installed extension
    fn load_from_extension(&self, ext_name: &str) -> ExtensionResult<(ExtensionConfig, PathBuf)> {
        let vx_paths = vx_paths::VxPaths::new().map_err(|e| ExtensionError::Io {
            message: format!("Failed to get vx paths: {}", e),
            path: None,
            source: std::io::Error::other(e.to_string()),
        })?;

        // Search in extensions directories
        let extensions_dir = vx_paths.base_dir.join("extensions");
        let extensions_dev_dir = vx_paths.base_dir.join("extensions-dev");

        let search_paths = [
            extensions_dev_dir.join(ext_name),
            extensions_dir.join(ext_name),
        ];

        for ext_path in &search_paths {
            let config_path = ext_path.join("vx-extension.toml");
            if config_path.exists() {
                let config = ExtensionConfig::from_file(&config_path)?;
                return Ok((config, config_path));
            }
        }

        Err(ExtensionError::ExtensionNotFound {
            name: ext_name.to_string(),
            available: vec![],
            searched_paths: search_paths.to_vec(),
        })
    }

    /// Load config from GitHub
    fn load_from_github(&self, github_ref: &str) -> ExtensionResult<(ExtensionConfig, PathBuf)> {
        // Parse github:user/repo/path[@version]
        let (repo_path, version) = if let Some(at_pos) = github_ref.rfind('@') {
            (&github_ref[..at_pos], Some(&github_ref[at_pos + 1..]))
        } else {
            (github_ref, None)
        };

        let parts: Vec<&str> = repo_path.splitn(3, '/').collect();
        if parts.len() < 2 {
            return Err(ExtensionError::RemoteInstallFailed {
                src: github_ref.to_string(),
                reason: "Invalid GitHub reference. Use: github:user/repo/path/to/config.toml"
                    .to_string(),
            });
        }

        let user = parts[0];
        let repo = parts[1];
        let file_path = if parts.len() > 2 {
            parts[2]
        } else {
            "vx-extension.toml"
        };

        let branch = version.unwrap_or("main");
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            user, repo, branch, file_path
        );

        self.load_from_url(&url)
    }

    /// Load config from URL
    fn load_from_url(&self, url: &str) -> ExtensionResult<(ExtensionConfig, PathBuf)> {
        // Create cache path based on URL hash
        let url_hash = format!("{:x}", md5_hash(url));
        let cache_path = self.cache_dir.join("configs").join(&url_hash);

        // Try to load from cache first
        if cache_path.exists() {
            if let Ok(config) = ExtensionConfig::from_file(&cache_path) {
                return Ok((config, cache_path));
            }
        }

        // Download the config
        let content = download_content(url).map_err(|e| ExtensionError::RemoteInstallFailed {
            src: url.to_string(),
            reason: e,
        })?;

        // Parse the config
        let config = ExtensionConfig::parse(&content, None)?;

        // Cache the config
        if let Some(parent) = cache_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&cache_path, &content);

        Ok((config, cache_path))
    }

    /// Merge two configs (child overrides parent)
    fn merge_configs(&self, parent: ExtensionConfig, child: ExtensionConfig) -> ExtensionConfig {
        ExtensionConfig {
            // Child metadata always wins
            extension: child.extension,

            // Merge runtime requirements
            runtime: if child.runtime.requires.is_some() {
                child.runtime
            } else {
                parent.runtime
            },

            // Merge entrypoint (child wins if set)
            entrypoint: if child.entrypoint.main.is_some() {
                child.entrypoint
            } else {
                crate::config::EntrypointConfig {
                    main: parent.entrypoint.main,
                    args: if child.entrypoint.args.is_empty() {
                        parent.entrypoint.args
                    } else {
                        child.entrypoint.args
                    },
                    arguments: if child.entrypoint.arguments.is_empty() {
                        parent.entrypoint.arguments
                    } else {
                        child.entrypoint.arguments
                    },
                }
            },

            // Merge commands (child commands override parent)
            commands: {
                let mut merged = parent.commands;
                merged.extend(child.commands);
                merged
            },

            // Merge hooks
            hooks: {
                let mut merged = parent.hooks;
                merged.extend(child.hooks);
                merged
            },

            // Merge env vars (child overrides parent)
            env: {
                let mut merged = parent.env;
                merged.extend(child.env);
                merged
            },

            // Don't propagate extends
            extends: None,
        }
    }
}

impl Default for ConfigInheritance {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            max_depth: 10,
            cache_dir: PathBuf::from(".vx/cache"),
        })
    }
}

/// Simple MD5-like hash for URL caching (not cryptographic)
fn md5_hash(input: &str) -> u64 {
    let mut hash: u64 = 0;
    for (i, byte) in input.bytes().enumerate() {
        hash = hash.wrapping_add((byte as u64).wrapping_mul((i as u64).wrapping_add(1)));
        hash = hash.rotate_left(5);
    }
    hash
}

/// Download content from URL (blocking, for simplicity)
fn download_content(url: &str) -> Result<String, String> {
    // Use a simple blocking HTTP client
    // In production, this should use async reqwest
    std::process::Command::new("curl")
        .args(["-fsSL", url])
        .output()
        .map_err(|e| format!("Failed to download: {}", e))
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8: {}", e))
            } else {
                Err(format!(
                    "Download failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_configs() {
        let resolver = ConfigInheritance::default();

        let parent = ExtensionConfig::parse(
            r#"
[extension]
name = "parent"

[env]
BASE_VAR = "from_parent"
OVERRIDE_VAR = "parent_value"

[commands.shared]
description = "Shared command"
script = "shared.py"
"#,
            None,
        )
        .unwrap();

        let child = ExtensionConfig::parse(
            r#"
[extension]
name = "child"

[env]
OVERRIDE_VAR = "child_value"
CHILD_VAR = "from_child"

[commands.custom]
description = "Custom command"
script = "custom.py"
"#,
            None,
        )
        .unwrap();

        let merged = resolver.merge_configs(parent, child);

        assert_eq!(merged.extension.name, "child");
        assert_eq!(merged.env.get("BASE_VAR"), Some(&"from_parent".to_string()));
        assert_eq!(
            merged.env.get("OVERRIDE_VAR"),
            Some(&"child_value".to_string())
        );
        assert_eq!(merged.env.get("CHILD_VAR"), Some(&"from_child".to_string()));
        assert!(merged.commands.contains_key("shared"));
        assert!(merged.commands.contains_key("custom"));
    }

    #[test]
    fn test_md5_hash() {
        let hash1 = md5_hash("https://example.com/config.toml");
        let hash2 = md5_hash("https://example.com/config.toml");
        let hash3 = md5_hash("https://different.com/config.toml");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
