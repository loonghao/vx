//! Remote development configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Remote development configuration (Phase 3)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct RemoteConfig {
    /// Enable remote development config generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// GitHub Codespaces configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codespaces: Option<CodespacesConfig>,

    /// GitPod configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitpod: Option<GitpodConfig>,

    /// DevContainer configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devcontainer: Option<DevContainerConfig>,
}

/// GitHub Codespaces configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct CodespacesConfig {
    /// Enable Codespaces config generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Machine type (basicLinux32gb, standardLinux32gb, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,

    /// Prebuild configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuild: Option<PrebuildConfig>,

    /// VS Code extensions to install
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<String>,

    /// Forwarded ports
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<PortForward>,
}

/// Prebuild configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PrebuildConfig {
    /// Enable prebuilds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Branches to prebuild
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<String>,
}

/// Port forwarding configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PortForward {
    /// Port number
    pub port: u16,

    /// Port label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Visibility (private, org, public)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    /// On auto-forward action (notify, openBrowser, ignore)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_auto_forward: Option<String>,
}

/// GitPod configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct GitpodConfig {
    /// Enable GitPod config generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Docker image to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Init tasks
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<GitpodTask>,

    /// VS Code extensions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<String>,

    /// Ports configuration
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<GitpodPort>,

    /// Prebuilds configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilds: Option<GitpodPrebuilds>,
}

/// GitPod task
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct GitpodTask {
    /// Task name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Init command (runs during prebuild)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init: Option<String>,

    /// Command (runs on workspace start)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Before command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
}

/// GitPod port configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct GitpodPort {
    /// Port number
    pub port: u16,

    /// Visibility (private, public)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    /// On open action (notify, open-browser, open-preview, ignore)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_open: Option<String>,
}

/// GitPod prebuilds configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct GitpodPrebuilds {
    /// Enable prebuilds for default branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master: Option<bool>,

    /// Enable prebuilds for branches
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branches: Option<bool>,

    /// Enable prebuilds for PRs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_requests: Option<bool>,

    /// Add check to PRs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_check: Option<bool>,
}

/// DevContainer configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DevContainerConfig {
    /// Enable devcontainer.json generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Container name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Docker image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Dockerfile path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<String>,

    /// Docker build context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Features to install
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub features: HashMap<String, serde_json::Value>,

    /// Post-create command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_create_command: Option<String>,

    /// Post-start command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_start_command: Option<String>,

    /// VS Code customizations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customizations: Option<DevContainerCustomizations>,

    /// Forwarded ports
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forward_ports: Vec<u16>,

    /// Remote user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_user: Option<String>,

    /// Container environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub container_env: HashMap<String, String>,

    /// Mounts
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mounts: Vec<String>,
}

/// DevContainer customizations
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DevContainerCustomizations {
    /// VS Code customizations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vscode: Option<VsCodeCustomizations>,
}

/// VS Code customizations
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct VsCodeCustomizations {
    /// Extensions to install
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<String>,

    /// Settings
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub settings: HashMap<String, serde_json::Value>,
}
