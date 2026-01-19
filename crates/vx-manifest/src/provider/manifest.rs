use crate::{Ecosystem, ManifestError, PlatformConstraint, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::runtime::RuntimeDef;

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
            .is_none_or(|c| c.is_current_platform_supported())
    }

    /// Get the platform constraint description
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
