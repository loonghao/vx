//! Python environment configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Python environment configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PythonConfig {
    /// Python version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Virtual environment path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venv: Option<String>,

    /// Package manager (uv, pip, poetry)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,

    /// Dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<PythonDependencies>,
}

/// Python dependencies
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PythonDependencies {
    /// Requirements files
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<String>,

    /// Direct packages
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,

    /// Git dependencies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub git: Vec<String>,

    /// Dev dependencies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dev: Vec<String>,
}
