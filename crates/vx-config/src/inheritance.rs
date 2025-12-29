//! Configuration inheritance system
//!
//! This module handles configuration inheritance from remote presets,
//! including fetching, merging, and version locking.

use crate::{ConfigError, ConfigResult, VxConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Configuration inheritance manager
pub struct InheritanceManager {
    /// Cache directory for remote presets
    #[allow(dead_code)]
    cache_dir: std::path::PathBuf,
}

/// Remote preset source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetSource {
    /// Preset URL or identifier
    pub url: String,
    /// Version constraint
    pub version: Option<String>,
    /// SHA256 hash for verification
    pub sha256: Option<String>,
}

/// Merge strategy for configuration inheritance
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    /// Child values override parent values
    #[default]
    Override,
    /// Merge maps, child takes precedence
    Merge,
    /// Append arrays
    Append,
    /// Use parent value if child is not set
    Default,
}

/// Version lock entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    /// Source URL
    pub url: String,
    /// Resolved version
    pub version: String,
    /// SHA256 hash
    pub sha256: String,
    /// Lock timestamp
    pub locked_at: String,
}

/// Version lock file content
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LockFile {
    /// Lock file version
    pub version: u32,
    /// Locked presets
    pub presets: HashMap<String, LockEntry>,
}

impl InheritanceManager {
    /// Create a new inheritance manager
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
        }
    }

    /// Resolve a preset URL to a local path
    pub fn resolve_preset_url(url: &str) -> PresetSource {
        // Handle shorthand formats
        let resolved_url = if url.starts_with("github:") {
            // github:owner/repo -> https://raw.githubusercontent.com/owner/repo/main/vx.toml
            let path = url.strip_prefix("github:").unwrap();
            format!("https://raw.githubusercontent.com/{}/main/vx.toml", path)
        } else if url.starts_with("vx:") {
            // vx:preset-name -> official vx preset
            let name = url.strip_prefix("vx:").unwrap();
            format!("https://vx.dev/presets/{}.toml", name)
        } else {
            url.to_string()
        };

        PresetSource {
            url: resolved_url,
            version: None,
            sha256: None,
        }
    }

    /// Parse extends URL with version constraint
    /// Format: "url@version" or "url#sha256"
    pub fn parse_extends(extends: &str) -> PresetSource {
        let (url, version, sha256) = if let Some((url, rest)) = extends.split_once('@') {
            if let Some((ver, hash)) = rest.split_once('#') {
                (url, Some(ver.to_string()), Some(hash.to_string()))
            } else {
                (url, Some(rest.to_string()), None)
            }
        } else if let Some((url, hash)) = extends.split_once('#') {
            (url, None, Some(hash.to_string()))
        } else {
            (extends, None, None)
        };

        let mut source = Self::resolve_preset_url(url);
        source.version = version;
        source.sha256 = sha256;
        source
    }

    /// Merge two configurations
    pub fn merge_configs(parent: &VxConfig, child: &VxConfig, strategy: MergeStrategy) -> VxConfig {
        let mut result = parent.clone();

        // Merge tools
        for (name, version) in &child.tools {
            result.tools.insert(name.clone(), version.clone());
        }

        // Merge scripts
        for (name, script) in &child.scripts {
            result.scripts.insert(name.clone(), script.clone());
        }

        // Merge services
        for (name, service) in &child.services {
            result.services.insert(name.clone(), service.clone());
        }

        // Override simple fields if set in child
        if child.min_version.is_some() {
            result.min_version = child.min_version.clone();
        }
        if child.project.is_some() {
            result.project = child.project.clone();
        }
        if child.python.is_some() {
            result.python = child.python.clone();
        }
        if child.env.is_some() {
            match strategy {
                MergeStrategy::Merge => {
                    if let (Some(parent_env), Some(child_env)) = (&result.env, &child.env) {
                        let mut merged = parent_env.vars.clone();
                        merged.extend(child_env.vars.clone());
                        result.env = Some(crate::EnvConfig {
                            vars: merged,
                            ..child_env.clone()
                        });
                    }
                }
                _ => {
                    result.env = child.env.clone();
                }
            }
        }
        if child.settings.is_some() {
            result.settings = child.settings.clone();
        }
        if child.hooks.is_some() {
            result.hooks = child.hooks.clone();
        }
        if child.dependencies.is_some() {
            result.dependencies = child.dependencies.clone();
        }

        // Phase 2+ fields
        if child.ai.is_some() {
            result.ai = child.ai.clone();
        }
        if child.docs.is_some() {
            result.docs = child.docs.clone();
        }

        // Phase 3+ fields
        if child.team.is_some() {
            result.team = child.team.clone();
        }
        if child.remote.is_some() {
            result.remote = child.remote.clone();
        }

        // Phase 4+ fields
        if child.security.is_some() {
            result.security = child.security.clone();
        }
        if child.test.is_some() {
            result.test = child.test.clone();
        }
        if child.telemetry.is_some() {
            result.telemetry = child.telemetry.clone();
        }

        // Phase 5+ fields
        if child.container.is_some() {
            result.container = child.container.clone();
        }
        if child.versioning.is_some() {
            result.versioning = child.versioning.clone();
        }

        result
    }

    /// Load lock file
    pub fn load_lock_file(path: impl AsRef<Path>) -> ConfigResult<LockFile> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::IoError(format!("Failed to read lock file: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse lock file: {}", e)))
    }

    /// Save lock file
    pub fn save_lock_file(path: impl AsRef<Path>, lock: &LockFile) -> ConfigResult<()> {
        let content = toml::to_string_pretty(lock).map_err(|e| {
            ConfigError::ParseError(format!("Failed to serialize lock file: {}", e))
        })?;

        std::fs::write(path.as_ref(), content)
            .map_err(|e| ConfigError::IoError(format!("Failed to write lock file: {}", e)))
    }

    /// Calculate SHA256 hash of content
    pub fn calculate_hash(content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify content against expected hash
    pub fn verify_hash(content: &str, expected: &str) -> bool {
        Self::calculate_hash(content) == expected
    }
}

/// Built-in presets
#[allow(dead_code)]
pub mod presets {
    /// Get built-in preset by name
    pub fn get_builtin(name: &str) -> Option<&'static str> {
        match name {
            "node" => Some(include_str!("../presets/node.toml")),
            "python" => Some(include_str!("../presets/python.toml")),
            "rust" => Some(include_str!("../presets/rust.toml")),
            "go" => Some(include_str!("../presets/go.toml")),
            "fullstack" => Some(include_str!("../presets/fullstack.toml")),
            _ => None,
        }
    }

    /// List available built-in presets
    pub fn list_builtin() -> Vec<&'static str> {
        vec!["node", "python", "rust", "go", "fullstack"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extends() {
        let source = InheritanceManager::parse_extends("github:vx-dev/presets@v1.0.0");
        assert!(source.url.contains("githubusercontent.com"));
        assert_eq!(source.version, Some("v1.0.0".to_string()));

        let source = InheritanceManager::parse_extends("vx:node");
        assert!(source.url.contains("vx.dev/presets"));
    }

    #[test]
    fn test_merge_configs() {
        let mut parent = VxConfig::default();
        parent.tools.insert(
            "node".to_string(),
            crate::ToolVersion::Simple("18".to_string()),
        );

        let mut child = VxConfig::default();
        child.tools.insert(
            "node".to_string(),
            crate::ToolVersion::Simple("20".to_string()),
        );
        child.tools.insert(
            "rust".to_string(),
            crate::ToolVersion::Simple("stable".to_string()),
        );

        let merged = InheritanceManager::merge_configs(&parent, &child, MergeStrategy::Override);
        assert_eq!(merged.get_tool_version("node"), Some("20".to_string()));
        assert_eq!(merged.get_tool_version("rust"), Some("stable".to_string()));
    }

    #[test]
    fn test_calculate_hash() {
        let hash = InheritanceManager::calculate_hash("test content");
        assert_eq!(hash.len(), 64); // SHA256 hex string
    }
}
