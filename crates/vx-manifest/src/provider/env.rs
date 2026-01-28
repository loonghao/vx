use crate::VersionRequest;
use serde::{Deserialize, Serialize};

/// Environment variable configuration
///
/// Supports static, dynamic (template), and conditional environment variables.
/// Template variables:
/// - `{install_dir}` - Installation directory
/// - `{version}` - Current version
/// - `{executable}` - Executable path
/// - `{PATH}` - Original PATH value
/// - `{env:VAR}` - Reference other environment variable
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EnvConfig {
    /// Static environment variables
    #[serde(default)]
    pub vars: std::collections::HashMap<String, String>,

    /// Conditional environment variables (version-based)
    /// Key is version constraint (e.g., ">=18"), value is env vars
    #[serde(default)]
    pub conditional: std::collections::HashMap<String, std::collections::HashMap<String, String>>,

    /// Advanced environment configuration (similar to rez)
    #[serde(default)]
    pub advanced: Option<AdvancedEnvConfig>,
}

/// Advanced environment configuration similar to rez
///
/// Supports PATH manipulation, environment inheritance control, and isolation.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AdvancedEnvConfig {
    /// PATH prepend entries (added before existing PATH)
    #[serde(default)]
    pub path_prepend: Vec<String>,

    /// PATH append entries (added after existing PATH)
    #[serde(default)]
    pub path_append: Vec<String>,

    /// Environment variables with special handling
    #[serde(default)]
    pub env_vars: std::collections::HashMap<String, EnvVarConfig>,

    /// Whether to isolate from system environment by default
    #[serde(default = "default_isolate_env")]
    pub isolate: bool,

    /// Which system environment variables to inherit when isolated
    #[serde(default)]
    pub inherit_system_vars: Vec<String>,
}

/// Configuration for individual environment variables
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EnvVarConfig {
    /// Simple string value
    Simple(String),
    /// Advanced configuration with prepend/append
    Advanced {
        /// Value to set
        value: Option<String>,
        /// Prepend to existing value (separated by OS-specific separator)
        prepend: Option<Vec<String>>,
        /// Append to existing value (separated by OS-specific separator)
        append: Option<Vec<String>>,
        /// Replace existing value entirely
        #[serde(default)]
        replace: bool,
    },
}

fn default_isolate_env() -> bool {
    true
}

impl EnvConfig {
    /// Get environment variables for a specific version
    pub fn get_vars_for_version(&self, version: &str) -> std::collections::HashMap<String, String> {
        let mut result = self.vars.clone();

        for (constraint, vars) in &self.conditional {
            let req = VersionRequest::parse(constraint);
            if req.satisfies(version) {
                result.extend(vars.clone());
            }
        }

        result
    }

    /// Check if there are any environment variables configured
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty() && self.conditional.is_empty() && self.advanced.is_none()
    }

    /// Check if environment isolation is enabled
    pub fn is_isolated(&self) -> bool {
        self.advanced.as_ref().map(|a| a.isolate).unwrap_or(true)
    }

    /// Get system variables to inherit when isolated
    pub fn inherit_system_vars(&self) -> &[String] {
        self.advanced
            .as_ref()
            .map(|a| &a.inherit_system_vars[..])
            .unwrap_or(&[])
    }
}
