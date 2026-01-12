use serde::{Deserialize, Serialize};

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
