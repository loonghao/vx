//! Telemetry configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Telemetry configuration (Phase 4)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Enable telemetry (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Anonymous mode (no identifiable data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymous: Option<bool>,

    /// Build time tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_tracking: Option<BuildTrackingConfig>,

    /// OTLP export configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otlp: Option<OtlpConfig>,

    /// Metrics to collect
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<String>,
}

/// Build time tracking configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct BuildTrackingConfig {
    /// Enable build tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Track tool install times
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_times: Option<bool>,

    /// Track script execution times
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_times: Option<bool>,

    /// Track service startup times
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_times: Option<bool>,

    /// Output file for local tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// OTLP export configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct OtlpConfig {
    /// Enable OTLP export
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// OTLP endpoint URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// Headers for authentication
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,

    /// Service name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,

    /// Export interval in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
}
