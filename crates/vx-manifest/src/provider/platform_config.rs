use serde::{Deserialize, Serialize};

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
