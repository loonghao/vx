use serde::{Deserialize, Serialize};

use super::defaults::{default_check_on, default_health_timeout};

/// Health check configuration
///
/// Validates that a runtime installation is working correctly.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthConfig {
    /// Command to check health (supports {executable} template)
    pub check_command: String,

    /// Expected output pattern (regex)
    #[serde(default)]
    pub expected_pattern: Option<String>,

    /// Expected exit code (if not specified, any exit code is accepted)
    #[serde(default)]
    pub exit_code: Option<i32>,

    /// Timeout in milliseconds
    #[serde(default = "default_health_timeout")]
    pub timeout_ms: u64,

    /// Optional verification script path
    #[serde(default)]
    pub verify_script: Option<String>,

    /// When to run health checks
    #[serde(default = "default_check_on")]
    pub check_on: Vec<String>,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_command: "{executable} --version".to_string(),
            expected_pattern: None,
            exit_code: Some(0),
            timeout_ms: 5000,
            verify_script: None,
            check_on: vec!["install".to_string()],
        }
    }
}
