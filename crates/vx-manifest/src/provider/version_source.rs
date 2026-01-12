use serde::{Deserialize, Serialize};

/// Version source configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionSourceDef {
    /// Source type (e.g., "github-releases", "npm", "pypi")
    pub source: String,
    /// GitHub owner (for github-releases)
    #[serde(default)]
    pub owner: Option<String>,
    /// GitHub repo (for github-releases)
    #[serde(default)]
    pub repo: Option<String>,
    /// Whether to strip 'v' prefix from versions
    #[serde(default)]
    pub strip_v_prefix: bool,
    /// LTS version pattern
    #[serde(default)]
    pub lts_pattern: Option<String>,
}
