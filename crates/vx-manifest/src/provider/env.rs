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
        self.vars.is_empty() && self.conditional.is_empty()
    }
}
