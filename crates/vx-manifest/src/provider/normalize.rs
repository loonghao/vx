//! Install Normalize Configuration (RFC 0022)
//!
//! This module defines the normalization configuration for post-install processing,
//! including renaming, moving, and creating symbolic links to standardize the
//! installation directory structure.
//!
//! ## Example provider.toml
//!
//! ```toml
//! [runtimes.normalize]
//! enabled = true
//!
//! [[runtimes.normalize.executables]]
//! source = "ImageMagick-*-Q16-HDRI/magick.exe"
//! target = "magick.exe"
//! action = "link"
//!
//! [[runtimes.normalize.aliases]]
//! name = "im"
//! target = "magick"
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Normalize action type
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NormalizeAction {
    /// Create symbolic link (default)
    #[default]
    Link,
    /// Create hard link
    #[serde(alias = "hard_link")]
    HardLink,
    /// Copy file/directory
    Copy,
    /// Move file/directory
    Move,
}

/// Executable normalization rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableNormalize {
    /// Source path pattern (supports glob and variables like {version}, {name}, *)
    pub source: String,

    /// Target path (relative to bin/ directory)
    pub target: String,

    /// Action to perform (default: link)
    #[serde(default)]
    pub action: NormalizeAction,

    /// File permissions (Unix only, e.g., "755")
    #[serde(default)]
    pub permissions: Option<String>,

    /// Only apply on specific platforms
    #[serde(default)]
    pub platforms: Option<Vec<String>>,
}

/// Directory normalization rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNormalize {
    /// Source directory pattern (supports glob and variables)
    pub source: String,

    /// Target directory (relative to install root)
    pub target: String,

    /// Action to perform (default: link)
    #[serde(default)]
    pub action: NormalizeAction,

    /// Only apply on specific platforms
    #[serde(default)]
    pub platforms: Option<Vec<String>>,
}

/// Alias/symlink definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasNormalize {
    /// Alias name (will be created in bin/)
    pub name: String,

    /// Target executable name (in bin/)
    pub target: String,

    /// Only apply on specific platforms
    #[serde(default)]
    pub platforms: Option<Vec<String>>,
}

/// Platform-specific normalize configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformNormalizeConfig {
    /// Executable normalization rules
    #[serde(default)]
    pub executables: Vec<ExecutableNormalize>,

    /// Directory normalization rules
    #[serde(default)]
    pub directories: Vec<DirectoryNormalize>,

    /// Aliases to create
    #[serde(default)]
    pub aliases: Vec<AliasNormalize>,
}

/// Normalize configuration for a runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizeConfig {
    /// Enable normalization (default: true when normalize section exists)
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Executable normalization rules (cross-platform)
    #[serde(default)]
    pub executables: Vec<ExecutableNormalize>,

    /// Directory normalization rules (cross-platform)
    #[serde(default)]
    pub directories: Vec<DirectoryNormalize>,

    /// Aliases to create (cross-platform)
    #[serde(default)]
    pub aliases: Vec<AliasNormalize>,

    /// Platform-specific configurations
    /// Keys: "windows", "macos", "linux", "unix" (linux + macos)
    #[serde(default)]
    pub platforms: HashMap<String, PlatformNormalizeConfig>,
}

impl Default for NormalizeConfig {
    fn default() -> Self {
        Self {
            enabled: true, // default to true
            executables: Vec::new(),
            directories: Vec::new(),
            aliases: Vec::new(),
            platforms: HashMap::new(),
        }
    }
}

fn default_enabled() -> bool {
    true
}

impl NormalizeConfig {
    /// Create a new empty normalize config
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if normalization is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the current platform key
    fn current_platform_key() -> &'static str {
        if cfg!(windows) {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        }
    }

    /// Check if a rule applies to current platform
    fn applies_to_current_platform(platforms: &Option<Vec<String>>) -> bool {
        match platforms {
            None => true, // No platform restriction
            Some(platforms) => {
                let current = Self::current_platform_key();
                platforms
                    .iter()
                    .any(|p| p == current || (p == "unix" && !cfg!(windows)))
            }
        }
    }

    /// Get effective configuration for current platform
    pub fn get_effective_config(&self) -> EffectiveNormalizeConfig {
        let platform_key = Self::current_platform_key();

        // Start with cross-platform rules (filtered by platform constraint)
        let mut executables: Vec<ExecutableNormalize> = self
            .executables
            .iter()
            .filter(|e| Self::applies_to_current_platform(&e.platforms))
            .cloned()
            .collect();

        let mut directories: Vec<DirectoryNormalize> = self
            .directories
            .iter()
            .filter(|d| Self::applies_to_current_platform(&d.platforms))
            .cloned()
            .collect();

        let mut aliases: Vec<AliasNormalize> = self
            .aliases
            .iter()
            .filter(|a| Self::applies_to_current_platform(&a.platforms))
            .cloned()
            .collect();

        // Merge platform-specific config
        if let Some(platform_config) = self.platforms.get(platform_key) {
            executables.extend(platform_config.executables.clone());
            directories.extend(platform_config.directories.clone());
            aliases.extend(platform_config.aliases.clone());
        }

        // Also check "unix" for linux/macos
        if !cfg!(windows) {
            if let Some(unix_config) = self.platforms.get("unix") {
                executables.extend(unix_config.executables.clone());
                directories.extend(unix_config.directories.clone());
                aliases.extend(unix_config.aliases.clone());
            }
        }

        EffectiveNormalizeConfig {
            enabled: self.enabled,
            executables,
            directories,
            aliases,
        }
    }

    /// Add an executable normalization rule
    pub fn add_executable(&mut self, source: &str, target: &str) -> &mut Self {
        self.executables.push(ExecutableNormalize {
            source: source.to_string(),
            target: target.to_string(),
            action: NormalizeAction::default(),
            permissions: None,
            platforms: None,
        });
        self
    }

    /// Add an alias
    pub fn add_alias(&mut self, name: &str, target: &str) -> &mut Self {
        self.aliases.push(AliasNormalize {
            name: name.to_string(),
            target: target.to_string(),
            platforms: None,
        });
        self
    }
}

/// Effective configuration after platform resolution
#[derive(Debug, Clone)]
pub struct EffectiveNormalizeConfig {
    /// Whether normalization is enabled
    pub enabled: bool,
    /// Executable normalization rules
    pub executables: Vec<ExecutableNormalize>,
    /// Directory normalization rules
    pub directories: Vec<DirectoryNormalize>,
    /// Aliases to create
    pub aliases: Vec<AliasNormalize>,
}

impl EffectiveNormalizeConfig {
    /// Check if there are any rules to apply
    pub fn has_rules(&self) -> bool {
        self.enabled
            && (!self.executables.is_empty()
                || !self.directories.is_empty()
                || !self.aliases.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_config_default() {
        let config = NormalizeConfig::default();
        assert!(config.enabled);
        assert!(config.executables.is_empty());
        assert!(config.directories.is_empty());
        assert!(config.aliases.is_empty());
    }

    #[test]
    fn test_effective_config_merges_platforms() {
        let mut config = NormalizeConfig::new();

        // Add cross-platform rule
        config.add_executable("bin/tool", "tool");

        // Add platform-specific rule
        let mut windows_config = PlatformNormalizeConfig::default();
        windows_config.executables.push(ExecutableNormalize {
            source: "tool.exe".to_string(),
            target: "tool.exe".to_string(),
            action: NormalizeAction::Link,
            permissions: None,
            platforms: None,
        });
        config
            .platforms
            .insert("windows".to_string(), windows_config);

        let effective = config.get_effective_config();

        #[cfg(windows)]
        assert_eq!(effective.executables.len(), 2);

        #[cfg(not(windows))]
        assert_eq!(effective.executables.len(), 1);
    }

    #[test]
    fn test_platform_filter() {
        let mut config = NormalizeConfig::new();

        // Rule only for Windows
        config.executables.push(ExecutableNormalize {
            source: "tool.exe".to_string(),
            target: "tool.exe".to_string(),
            action: NormalizeAction::Link,
            permissions: None,
            platforms: Some(vec!["windows".to_string()]),
        });

        // Rule only for Unix
        config.executables.push(ExecutableNormalize {
            source: "tool".to_string(),
            target: "tool".to_string(),
            action: NormalizeAction::Link,
            permissions: Some("755".to_string()),
            platforms: Some(vec!["unix".to_string()]),
        });

        let effective = config.get_effective_config();

        #[cfg(windows)]
        {
            assert_eq!(effective.executables.len(), 1);
            assert_eq!(effective.executables[0].source, "tool.exe");
        }

        #[cfg(not(windows))]
        {
            assert_eq!(effective.executables.len(), 1);
            assert_eq!(effective.executables[0].source, "tool");
        }
    }

    #[test]
    fn test_action_deserialization() {
        let toml_str = r#"
            source = "bin/tool"
            target = "tool"
            action = "hard_link"
        "#;

        let rule: ExecutableNormalize = toml::from_str(toml_str).unwrap();
        assert_eq!(rule.action, NormalizeAction::HardLink);
    }
}
