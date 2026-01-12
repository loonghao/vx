use serde::{Deserialize, Serialize};

use super::defaults::default_exit_codes;


/// Version detection configuration
///
/// Declares how to detect installed versions of a runtime.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DetectionConfig {
    /// Command to get version (supports {executable} template)
    pub command: String,

    /// Regex pattern to extract version (capture group 1 is version)
    pub pattern: String,

    /// System paths to check for existing installations
    #[serde(default)]
    pub system_paths: Vec<String>,

    /// Environment variable hints (may indicate installation)
    #[serde(default)]
    pub env_hints: Vec<String>,

    /// Windows registry paths to check
    #[serde(default)]
    pub registry_paths: Vec<String>,

    /// Acceptable exit codes (default: [0])
    /// Some tools return non-zero exit codes even on success (e.g., cl.exe returns 2)
    #[serde(default = "default_exit_codes")]
    pub exit_codes: Vec<i32>,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            command: "{executable} --version".to_string(),
            pattern: r"v?(\d+\.\d+\.\d+)".to_string(),
            system_paths: Vec::new(),
            env_hints: Vec::new(),
            registry_paths: Vec::new(),
            exit_codes: default_exit_codes(),
        }
    }
}
