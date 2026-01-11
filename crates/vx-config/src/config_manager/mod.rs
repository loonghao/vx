//! Unified TOML Configuration Management
//!
//! This module provides a unified interface for managing TOML configuration files
//! including `vx.toml`, `provider.toml`, and other configuration files.
//!
//! ## Architecture
//!
//! - [`TomlConfig`]: Trait that all configuration types must implement
//! - [`TomlWriter`]: Builder for generating valid TOML output
//! - [`ConfigManager`]: Generic manager for CRUD operations on configurations
//!
//! ## Example
//!
//! ```rust,ignore
//! use vx_config::config_manager::{ConfigManager, TomlConfig};
//!
//! // Load configuration
//! let manager = ConfigManager::<VxConfig>::load("vx.toml")?;
//!
//! // Access the config
//! let config = manager.config();
//!
//! // Modify and save
//! manager.config_mut().tools.insert("node".into(), "20".into());
//! manager.save()?;
//! ```

mod toml_writer;
mod traits;

pub use toml_writer::{
    escape_toml_key, escape_toml_string, format_toml_kv, TomlDocument, TomlWriter,
};
pub use traits::{
    ConfigVersion, TomlConfig, ValidationIssue, ValidationResult, ValidationSeverity,
};

use crate::error::{ConfigError, ConfigResult};
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;

/// Generic configuration manager for TOML files
///
/// Provides unified CRUD operations for any configuration type that
/// implements the [`TomlConfig`] trait.
///
/// ## Format Preservation
///
/// When loading from a file, `ConfigManager` preserves the original TOML document
/// structure including comments and formatting. Use `save_preserving_format()` to
/// write changes while keeping the original formatting intact.
#[derive(Debug)]
pub struct ConfigManager<T: TomlConfig> {
    /// The parsed configuration
    config: T,
    /// Path to the configuration file (if loaded from file)
    path: Option<PathBuf>,
    /// Original content (for preserving comments and formatting)
    original_content: Option<String>,
    /// Parsed document for format-preserving edits
    document: Option<DocumentMut>,
}

impl<T: TomlConfig + DeserializeOwned + Serialize + Default> ConfigManager<T> {
    /// Create a new manager with default configuration
    pub fn new() -> Self {
        Self {
            config: T::default(),
            path: None,
            original_content: None,
            document: None,
        }
    }

    /// Load configuration from a file
    pub fn load<P: AsRef<Path>>(path: P) -> ConfigResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::NotFound {
                path: path.display().to_string(),
            });
        }

        let content = fs::read_to_string(path)?;
        let config: T = toml::from_str(&content)?;

        // Parse document for format-preserving edits
        let document = content.parse::<DocumentMut>().ok();

        Ok(Self {
            config,
            path: Some(path.to_path_buf()),
            original_content: Some(content),
            document,
        })
    }

    /// Load configuration or create default if not exists
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> ConfigResult<Self> {
        let path = path.as_ref();

        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self {
                config: T::default(),
                path: Some(path.to_path_buf()),
                original_content: None,
                document: None,
            })
        }
    }

    /// Parse configuration from a string
    pub fn parse_str(content: &str) -> ConfigResult<Self> {
        let config: T = toml::from_str(content)?;
        let document = content.parse::<DocumentMut>().ok();

        Ok(Self {
            config,
            path: None,
            original_content: Some(content.to_string()),
            document,
        })
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &T {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut T {
        &mut self.config
    }

    /// Get the file path (if loaded from file)
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Set the file path
    pub fn set_path<P: AsRef<Path>>(&mut self, path: P) {
        self.path = Some(path.as_ref().to_path_buf());
    }

    /// Get the original content (if loaded from file)
    pub fn original_content(&self) -> Option<&str> {
        self.original_content.as_deref()
    }

    /// Check if we have a parsed document for format-preserving edits
    pub fn has_document(&self) -> bool {
        self.document.is_some()
    }

    /// Get access to the underlying document for format-preserving edits
    ///
    /// Returns `None` if the configuration was not loaded from a file or
    /// if the original content could not be parsed as a valid TOML document.
    pub fn document(&self) -> Option<&DocumentMut> {
        self.document.as_ref()
    }

    /// Get mutable access to the underlying document for format-preserving edits
    pub fn document_mut(&mut self) -> Option<&mut DocumentMut> {
        self.document.as_mut()
    }

    /// Validate the configuration
    pub fn validate(&self) -> ValidationResult {
        self.config.validate()
    }

    /// Check if configuration is valid
    pub fn is_valid(&self) -> bool {
        self.validate().is_valid()
    }

    /// Save configuration to the original path
    pub fn save(&self) -> ConfigResult<()> {
        let path = self.path.as_ref().ok_or_else(|| ConfigError::NotFound {
            path: "No path set for configuration".to_string(),
        })?;

        self.save_to(path)
    }

    /// Save configuration to a specific path
    ///
    /// This uses `toml::to_string_pretty` which generates clean but
    /// does not preserve original formatting or comments.
    /// Use `save_preserving_format()` to preserve formatting.
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> ConfigResult<()> {
        let content = self.config.to_toml_string()?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Save configuration while preserving original formatting and comments
    ///
    /// If a format-preserving document is available, this will update values
    /// in the document while keeping comments and formatting intact.
    /// Falls back to regular save if no document is available.
    pub fn save_preserving_format(&self) -> ConfigResult<()> {
        let path = self.path.as_ref().ok_or_else(|| ConfigError::NotFound {
            path: "No path set for configuration".to_string(),
        })?;

        self.save_preserving_format_to(path)
    }

    /// Save configuration to a specific path while preserving formatting
    pub fn save_preserving_format_to<P: AsRef<Path>>(&self, path: P) -> ConfigResult<()> {
        if let Some(doc) = &self.document {
            // Use the format-preserving document (DocumentMut::to_string is valid)
            fs::write(path, doc.to_string())?;
        } else {
            // Fall back to regular save
            self.save_to(path)?;
        }
        Ok(())
    }

    /// Generate TOML string from configuration
    pub fn to_toml_string(&self) -> ConfigResult<String> {
        self.config.to_toml_string()
    }

    /// Generate TOML string preserving original formatting
    ///
    /// If a format-preserving document is available, returns that.
    /// Otherwise falls back to `to_toml_string()`.
    pub fn to_toml_string_preserving_format(&self) -> ConfigResult<String> {
        if let Some(doc) = &self.document {
            // DocumentMut::to_string is valid from toml_edit
            Ok(doc.to_string())
        } else {
            self.to_toml_string()
        }
    }

    /// Reload configuration from disk
    ///
    /// This is useful after external modifications to the file.
    pub fn reload(&mut self) -> ConfigResult<()> {
        let path = self.path.clone().ok_or_else(|| ConfigError::NotFound {
            path: "No path set for configuration".to_string(),
        })?;

        let reloaded = Self::load(&path)?;
        self.config = reloaded.config;
        self.original_content = reloaded.original_content;
        self.document = reloaded.document;
        Ok(())
    }
}

impl<T: TomlConfig + DeserializeOwned + Serialize + Default> Default for ConfigManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Simple test config for testing ConfigManager
    #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
    struct TestConfig {
        #[serde(default)]
        name: String,
        #[serde(default)]
        version: String,
    }

    impl TomlConfig for TestConfig {
        fn config_name() -> &'static str {
            "test"
        }

        fn default_filename() -> &'static str {
            "test.toml"
        }
    }

    #[test]
    fn test_new_creates_default() {
        let manager: ConfigManager<TestConfig> = ConfigManager::new();
        assert_eq!(manager.config().name, "");
        assert!(manager.path().is_none());
        assert!(!manager.has_document());
    }

    #[test]
    fn test_load_from_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.toml");

        // Write test file with comments
        let content = r#"# Test configuration
name = "test-project"
version = "1.0.0"
"#;
        fs::write(&path, content).unwrap();

        let manager: ConfigManager<TestConfig> = ConfigManager::load(&path).unwrap();

        assert_eq!(manager.config().name, "test-project");
        assert_eq!(manager.config().version, "1.0.0");
        assert!(manager.has_document());
        assert!(manager
            .original_content()
            .unwrap()
            .contains("# Test configuration"));
    }

    #[test]
    fn test_load_or_default_existing() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.toml");

        fs::write(&path, "name = \"existing\"").unwrap();

        let manager: ConfigManager<TestConfig> = ConfigManager::load_or_default(&path).unwrap();
        assert_eq!(manager.config().name, "existing");
    }

    #[test]
    fn test_load_or_default_missing() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("nonexistent.toml");

        let manager: ConfigManager<TestConfig> = ConfigManager::load_or_default(&path).unwrap();
        assert_eq!(manager.config().name, "");
        assert_eq!(manager.path().unwrap(), path);
    }

    #[test]
    fn test_save_and_reload() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.toml");

        let mut manager: ConfigManager<TestConfig> = ConfigManager::new();
        manager.set_path(&path);
        manager.config_mut().name = "saved".to_string();
        manager.config_mut().version = "2.0.0".to_string();
        manager.save().unwrap();

        // Reload and verify
        let loaded: ConfigManager<TestConfig> = ConfigManager::load(&path).unwrap();
        assert_eq!(loaded.config().name, "saved");
        assert_eq!(loaded.config().version, "2.0.0");
    }

    #[test]
    fn test_format_preservation() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.toml");

        // Write file with specific formatting and comments
        let original = r#"# Project configuration
# This is important
name = "my-project"

# Version info
version = "1.0.0"
"#;
        fs::write(&path, original).unwrap();

        let manager: ConfigManager<TestConfig> = ConfigManager::load(&path).unwrap();
        manager.save_preserving_format().unwrap();

        // Read back and check comments are preserved
        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.contains("# Project configuration"));
        assert!(saved.contains("# This is important"));
        assert!(saved.contains("# Version info"));
    }

    #[test]
    fn test_reload() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.toml");

        fs::write(&path, "name = \"original\"").unwrap();

        let mut manager: ConfigManager<TestConfig> = ConfigManager::load(&path).unwrap();
        assert_eq!(manager.config().name, "original");

        // External modification
        fs::write(&path, "name = \"modified\"").unwrap();

        // Reload
        manager.reload().unwrap();
        assert_eq!(manager.config().name, "modified");
    }

    #[test]
    fn test_parse_str() {
        let content = "name = \"parsed\"\nversion = \"3.0.0\"";
        let manager: ConfigManager<TestConfig> = ConfigManager::parse_str(content).unwrap();

        assert_eq!(manager.config().name, "parsed");
        assert_eq!(manager.config().version, "3.0.0");
        assert!(manager.has_document());
    }

    #[test]
    fn test_load_nonexistent_returns_error() {
        let result: ConfigResult<ConfigManager<TestConfig>> =
            ConfigManager::load("/nonexistent/path/test.toml");
        assert!(result.is_err());
    }
}
