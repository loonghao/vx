//! Post-install normalization and mirror configuration types.
//!
//! These types were previously defined in `vx-manifest` and referenced via
//! `vx_manifest::NormalizeConfig` / `vx_manifest::MirrorConfig`.  They now
//! live here so that `vx-runtime-core` (and every provider crate) no longer
//! needs to depend on the TOML-parsing `vx-manifest` crate.
//!
//! The `provider.star` Starlark layer is responsible for producing these
//! values at runtime; `vx-manifest` / `provider.toml` parsing code in
//! `vx-runtime` converts the TOML representation into these types via `From`
//! implementations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// NormalizeConfig and related types (RFC 0022)
// ─────────────────────────────────────────────────────────────────────────────

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

/// Normalize configuration for a runtime (RFC 0022)
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
            enabled: true,
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
            None => true,
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

        if let Some(platform_config) = self.platforms.get(platform_key) {
            executables.extend(platform_config.executables.clone());
            directories.extend(platform_config.directories.clone());
            aliases.extend(platform_config.aliases.clone());
        }

        if !cfg!(windows)
            && let Some(unix_config) = self.platforms.get("unix")
        {
            executables.extend(unix_config.executables.clone());
            directories.extend(unix_config.directories.clone());
            aliases.extend(unix_config.aliases.clone());
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

// ─────────────────────────────────────────────────────────────────────────────
// MirrorConfig and related types (RFC 0018)
// ─────────────────────────────────────────────────────────────────────────────

/// Mirror configuration for alternative download sources (RFC 0018)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MirrorConfig {
    /// Mirror name (e.g., "taobao", "ustc")
    pub name: String,
    /// Geographic region (e.g., "cn", "us", "eu")
    #[serde(default)]
    pub region: Option<String>,
    /// Mirror URL
    pub url: String,
    /// Priority (higher = preferred)
    #[serde(default)]
    pub priority: i32,
    /// Whether this mirror is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}
