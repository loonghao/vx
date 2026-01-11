//! Configuration traits
//!
//! Defines the core traits that all TOML configuration types must implement.

use crate::error::ConfigResult;

/// Configuration version for migration support
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConfigVersion {
    pub major: u32,
    pub minor: u32,
}

impl ConfigVersion {
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    pub const V1: Self = Self::new(1, 0);
    pub const V2: Self = Self::new(2, 0);
}

impl std::fmt::Display for ConfigVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Validation issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// A single validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub message: String,
    pub path: Option<String>,
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            message: message.into(),
            path: None,
            suggestion: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            message: message.into(),
            path: None,
            suggestion: None,
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Result of configuration validation
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Warning)
    }

    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Warning)
    }

    pub fn add_error(&mut self, message: impl Into<String>) {
        self.issues.push(ValidationIssue::error(message));
    }

    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.issues.push(ValidationIssue::warning(message));
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.issues.extend(other.issues);
    }
}

/// Core trait for TOML configuration types
///
/// All configuration types that can be managed by [`ConfigManager`] must
/// implement this trait.
pub trait TomlConfig: Sized {
    /// Get the configuration type name (e.g., "vx", "provider", "extension")
    fn config_name() -> &'static str;

    /// Get the default filename (e.g., "vx.toml", "provider.toml")
    fn default_filename() -> &'static str;

    /// Get the current schema version
    fn schema_version() -> ConfigVersion {
        ConfigVersion::V1
    }

    /// Validate the configuration
    fn validate(&self) -> ValidationResult {
        ValidationResult::new()
    }

    /// Convert to TOML string
    fn to_toml_string(&self) -> ConfigResult<String>
    where
        Self: serde::Serialize,
    {
        Ok(toml::to_string_pretty(self)?)
    }

    /// Check if migration is needed
    fn needs_migration(&self) -> bool {
        false
    }

    /// Get detected version from content
    fn detect_version(_content: &str) -> ConfigVersion {
        ConfigVersion::V1
    }
}

