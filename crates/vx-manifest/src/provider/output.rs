use serde::{Deserialize, Serialize};

/// Output format configuration
///
/// Allows providers to customize how version lists, status, and other
/// output is formatted. Follows Unix text stream philosophy.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OutputConfig {
    /// Format string for version list display
    /// Template variables: {version}, {lts}, {installed}, {date}, {channel}
    #[serde(default)]
    pub list_format: Option<String>,

    /// Format string for status display
    /// Template variables: {name}, {version}, {path}, {source}
    #[serde(default)]
    pub status_format: Option<String>,

    /// Supported output formats
    #[serde(default)]
    pub formats: Vec<String>,

    /// Default output format (text, json, csv, table)
    #[serde(default)]
    pub default_format: Option<String>,

    /// Machine-readable flags for commands
    #[serde(default)]
    pub machine_flags: Option<MachineFlagsConfig>,

    /// Color configuration for terminal output
    #[serde(default)]
    pub colors: Option<OutputColorConfig>,
}

impl OutputConfig {
    /// Get the list format or a default
    pub fn list_format_or_default(&self) -> &str {
        self.list_format
            .as_deref()
            .unwrap_or("{version:>12} {installed:>10}")
    }

    /// Get the status format or a default
    pub fn status_format_or_default(&self) -> &str {
        self.status_format.as_deref().unwrap_or("{name} {version}")
    }

    /// Check if a format is supported
    pub fn supports_format(&self, format: &str) -> bool {
        if self.formats.is_empty() {
            // Default supported formats
            matches!(format, "text" | "json")
        } else {
            self.formats.iter().any(|f| f == format)
        }
    }
}

/// Machine-readable output flags
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MachineFlagsConfig {
    /// Flag for list command (e.g., "--json")
    #[serde(default)]
    pub list: Option<String>,

    /// Flag for info command
    #[serde(default)]
    pub info: Option<String>,

    /// Flag for status command
    #[serde(default)]
    pub status: Option<String>,
}

/// Color configuration for terminal output
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OutputColorConfig {
    /// Color for LTS versions
    #[serde(default)]
    pub lts: Option<String>,

    /// Color for current/active version
    #[serde(default)]
    pub current: Option<String>,

    /// Color for installed versions
    #[serde(default)]
    pub installed: Option<String>,

    /// Color for outdated versions
    #[serde(default)]
    pub outdated: Option<String>,

    /// Color for error messages
    #[serde(default)]
    pub error: Option<String>,
}
