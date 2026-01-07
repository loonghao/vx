//! Provider manifest types

use crate::{Ecosystem, ManifestError, PlatformConstraint, Result, VersionRequest};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Provider manifest - the root structure of provider.toml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderManifest {
    /// Provider metadata
    pub provider: ProviderMeta,
    /// Runtime definitions
    #[serde(default)]
    pub runtimes: Vec<RuntimeDef>,
}

impl ProviderManifest {
    /// Load a manifest from a file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse a manifest from TOML string
    pub fn parse(content: &str) -> Result<Self> {
        let manifest: Self = toml::from_str(content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the manifest
    fn validate(&self) -> Result<()> {
        if self.provider.name.is_empty() {
            return Err(ManifestError::MissingField("provider.name".to_string()));
        }

        for runtime in &self.runtimes {
            if runtime.name.is_empty() {
                return Err(ManifestError::MissingField("runtimes[].name".to_string()));
            }
            if runtime.executable.is_empty() {
                return Err(ManifestError::MissingField(format!(
                    "runtimes[{}].executable",
                    runtime.name
                )));
            }
        }

        Ok(())
    }

    /// Get a runtime definition by name
    pub fn get_runtime(&self, name: &str) -> Option<&RuntimeDef> {
        self.runtimes
            .iter()
            .find(|r| r.name == name || r.aliases.iter().any(|a| a == name))
    }

    /// Check if the provider is supported on the current platform
    pub fn is_current_platform_supported(&self) -> bool {
        self.provider.is_current_platform_supported()
    }

    /// Get the platform constraint description for the provider
    pub fn platform_description(&self) -> Option<String> {
        self.provider.platform_description()
    }

    /// Get a short platform label for display
    pub fn platform_label(&self) -> Option<String> {
        self.provider.platform_label()
    }

    /// Get all runtimes supported on the current platform
    pub fn supported_runtimes(&self) -> Vec<&RuntimeDef> {
        // If provider itself is not supported, return empty
        if !self.is_current_platform_supported() {
            return vec![];
        }

        self.runtimes
            .iter()
            .filter(|r| r.is_current_platform_supported())
            .collect()
    }
}

/// Provider metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderMeta {
    /// Provider name (required)
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,
    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,
    /// Ecosystem this provider belongs to
    #[serde(default)]
    pub ecosystem: Option<Ecosystem>,
    /// Platform constraints for the entire provider
    #[serde(default, rename = "platforms")]
    pub platform_constraint: Option<PlatformConstraint>,
}

impl ProviderMeta {
    /// Check if the provider is supported on the current platform
    pub fn is_current_platform_supported(&self) -> bool {
        self.platform_constraint
            .as_ref()
            .map_or(true, |c| c.is_current_platform_supported())
    }

    /// Get the platform constraint description
    pub fn platform_description(&self) -> Option<String> {
        self.platform_constraint.as_ref().and_then(|c| c.description())
    }

    /// Get a short platform label for display
    pub fn platform_label(&self) -> Option<String> {
        self.platform_constraint.as_ref().and_then(|c| c.short_label())
    }
}

/// Runtime definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeDef {
    /// Runtime name (required)
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Executable name (required)
    pub executable: String,
    /// Aliases for this runtime
    #[serde(default)]
    pub aliases: Vec<String>,
    /// If this runtime is bundled with another
    #[serde(default)]
    pub bundled_with: Option<String>,
    /// Dependency constraints
    #[serde(default)]
    pub constraints: Vec<ConstraintRule>,
    /// Hooks configuration
    #[serde(default)]
    pub hooks: Option<HooksDef>,
    /// Platform-specific configuration (download URLs, extensions, etc.)
    #[serde(default)]
    pub platforms: Option<PlatformsDef>,
    /// Platform constraints for this runtime
    #[serde(default, rename = "platform_constraint")]
    pub platform_constraint: Option<PlatformConstraint>,
    /// Version source configuration
    #[serde(default)]
    pub versions: Option<VersionSourceDef>,
    /// Executable configuration
    #[serde(default, rename = "executable_config")]
    pub executable_config: Option<ExecutableConfig>,
}

impl RuntimeDef {
    /// Get constraints that apply to a specific version
    pub fn get_constraints_for_version(&self, version: &str) -> Vec<&ConstraintRule> {
        self.constraints
            .iter()
            .filter(|c| c.matches(version))
            .collect()
    }

    /// Get all required dependencies for a specific version
    pub fn get_dependencies_for_version(&self, version: &str) -> Vec<&DependencyDef> {
        self.get_constraints_for_version(version)
            .into_iter()
            .flat_map(|c| c.requires.iter())
            .collect()
    }

    /// Get all recommended dependencies for a specific version
    pub fn get_recommendations_for_version(&self, version: &str) -> Vec<&DependencyDef> {
        self.get_constraints_for_version(version)
            .into_iter()
            .flat_map(|c| c.recommends.iter())
            .collect()
    }

    /// Check if the runtime is supported on the current platform
    pub fn is_current_platform_supported(&self) -> bool {
        self.platform_constraint
            .as_ref()
            .map_or(true, |c| c.is_current_platform_supported())
    }

    /// Get the platform constraint description
    pub fn platform_description(&self) -> Option<String> {
        self.platform_constraint.as_ref().and_then(|c| c.description())
    }

    /// Get a short platform label for display
    pub fn platform_label(&self) -> Option<String> {
        self.platform_constraint.as_ref().and_then(|c| c.short_label())
    }
}

/// Constraint rule - defines dependencies for a version range
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConstraintRule {
    /// Version condition (semver syntax)
    /// Examples: "^1", ">=2, <4", "*"
    pub when: String,
    /// Platform condition (optional)
    #[serde(default)]
    pub platform: Option<String>,
    /// Required dependencies
    #[serde(default)]
    pub requires: Vec<DependencyDef>,
    /// Recommended dependencies (optional, not enforced)
    #[serde(default)]
    pub recommends: Vec<DependencyDef>,
}

impl ConstraintRule {
    /// Check if this rule applies to the given version
    pub fn matches(&self, version: &str) -> bool {
        let req = VersionRequest::parse(&self.when);
        req.satisfies(version)
    }

    /// Check if this rule applies to the given platform
    pub fn matches_platform(&self, platform: &str) -> bool {
        match &self.platform {
            Some(p) => p == platform || p == "*",
            None => true, // No platform restriction
        }
    }
}

/// Dependency definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DependencyDef {
    /// Runtime name of the dependency
    pub runtime: String,
    /// Version constraint (semver syntax)
    pub version: String,
    /// Recommended version to install if none available
    #[serde(default)]
    pub recommended: Option<String>,
    /// Reason for this dependency
    #[serde(default)]
    pub reason: Option<String>,
    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

impl DependencyDef {
    /// Check if a version satisfies this dependency constraint
    pub fn satisfies(&self, version: &str) -> bool {
        let req = VersionRequest::parse(&self.version);
        req.satisfies(version)
    }
}

/// Hooks configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HooksDef {
    /// Hooks to run before executing the runtime
    #[serde(default)]
    pub pre_run: Vec<String>,
    /// Hooks to run after installation
    #[serde(default)]
    pub post_install: Vec<String>,
}

/// Platform-specific configurations
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlatformsDef {
    /// Windows-specific configuration
    #[serde(default)]
    pub windows: Option<PlatformConfig>,
    /// macOS-specific configuration
    #[serde(default)]
    pub macos: Option<PlatformConfig>,
    /// Linux-specific configuration
    #[serde(default)]
    pub linux: Option<PlatformConfig>,
    /// Unix (macOS + Linux) configuration
    #[serde(default)]
    pub unix: Option<PlatformConfig>,
}

/// Platform-specific configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlatformConfig {
    /// Executable extensions for this platform
    #[serde(default)]
    pub executable_extensions: Vec<String>,
    /// Download URL pattern for this platform
    #[serde(default)]
    pub download_url_pattern: Option<String>,
}

/// Version source configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionSourceDef {
    /// Source type (e.g., "github-releases", "npm", "pypi")
    pub source: String,
    /// GitHub owner (for github-releases)
    #[serde(default)]
    pub owner: Option<String>,
    /// GitHub repo (for github-releases)
    #[serde(default)]
    pub repo: Option<String>,
    /// Whether to strip 'v' prefix from versions
    #[serde(default)]
    pub strip_v_prefix: bool,
    /// LTS version pattern
    #[serde(default)]
    pub lts_pattern: Option<String>,
}

/// Executable configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExecutableConfig {
    /// Executable extensions (e.g., [".cmd", ".exe"])
    #[serde(default)]
    pub extensions: Vec<String>,
    /// Directory pattern after extraction
    #[serde(default)]
    pub dir_pattern: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test-runtime"
executable = "test"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        assert_eq!(manifest.provider.name, "test");
        assert_eq!(manifest.runtimes.len(), 1);
        assert_eq!(manifest.runtimes[0].name, "test-runtime");
    }

    #[test]
    fn test_parse_full_manifest() {
        let toml = r#"
[provider]
name = "yarn"
description = "Fast, reliable, and secure dependency management"
homepage = "https://yarnpkg.com"
ecosystem = "nodejs"

[[runtimes]]
name = "yarn"
description = "Yarn package manager"
executable = "yarn"
aliases = ["yarnpkg"]

[[runtimes.constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=12, <23", reason = "Yarn 1.x requires Node.js 12-22" }
]

[[runtimes.constraints]]
when = ">=4"
requires = [
    { runtime = "node", version = ">=18", recommended = "22" }
]
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        assert_eq!(manifest.provider.name, "yarn");
        assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::NodeJs));

        let runtime = &manifest.runtimes[0];
        assert_eq!(runtime.name, "yarn");
        assert_eq!(runtime.aliases, vec!["yarnpkg"]);
        assert_eq!(runtime.constraints.len(), 2);

        // Test constraint matching
        let v1_constraints = runtime.get_constraints_for_version("1.22.22");
        assert_eq!(v1_constraints.len(), 1);
        assert_eq!(v1_constraints[0].requires.len(), 1);
        assert_eq!(v1_constraints[0].requires[0].runtime, "node");

        let v4_constraints = runtime.get_constraints_for_version("4.0.0");
        assert_eq!(v4_constraints.len(), 1);
        assert_eq!(v4_constraints[0].requires[0].version, ">=18");
    }

    #[test]
    fn test_parse_manifest_with_platform_constraint() {
        let toml = r#"
[provider]
name = "msvc"
description = "Microsoft Visual C++ Compiler"
ecosystem = "system"

[provider.platforms]
os = ["windows"]

[[runtimes]]
name = "cl"
description = "MSVC C/C++ Compiler"
executable = "cl"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        assert_eq!(manifest.provider.name, "msvc");

        // Check platform constraint
        let platform_constraint = manifest.provider.platform_constraint.as_ref().unwrap();
        assert_eq!(platform_constraint.os.len(), 1);

        // Platform description
        assert_eq!(
            manifest.platform_description(),
            Some("Windows only".to_string())
        );
        assert_eq!(manifest.platform_label(), Some("Windows".to_string()));
    }

    #[test]
    fn test_parse_runtime_with_platform_constraint() {
        let toml = r#"
[provider]
name = "xcode"
description = "Apple Xcode Command Line Tools"

[[runtimes]]
name = "xcodebuild"
executable = "xcodebuild"

[runtimes.platform_constraint]
os = ["macos"]
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let runtime = &manifest.runtimes[0];

        // Check runtime platform constraint
        let platform_constraint = runtime.platform_constraint.as_ref().unwrap();
        assert_eq!(platform_constraint.os.len(), 1);
        assert_eq!(runtime.platform_description(), Some("macOS only".to_string()));
    }

    #[test]
    fn test_supported_runtimes() {
        let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "cross-platform"
executable = "cross"

[[runtimes]]
name = "windows-only"
executable = "win"

[runtimes.platform_constraint]
os = ["windows"]

[[runtimes]]
name = "macos-only"
executable = "mac"

[runtimes.platform_constraint]
os = ["macos"]
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();

        // Get supported runtimes for current platform
        let supported = manifest.supported_runtimes();

        // cross-platform should always be included
        assert!(supported.iter().any(|r| r.name == "cross-platform"));

        // Platform-specific runtimes depend on current OS
        #[cfg(target_os = "windows")]
        {
            assert!(supported.iter().any(|r| r.name == "windows-only"));
            assert!(!supported.iter().any(|r| r.name == "macos-only"));
        }

        #[cfg(target_os = "macos")]
        {
            assert!(!supported.iter().any(|r| r.name == "windows-only"));
            assert!(supported.iter().any(|r| r.name == "macos-only"));
        }

        #[cfg(target_os = "linux")]
        {
            assert!(!supported.iter().any(|r| r.name == "windows-only"));
            assert!(!supported.iter().any(|r| r.name == "macos-only"));
        }
    }
}
