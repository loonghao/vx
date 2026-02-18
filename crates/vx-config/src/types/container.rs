//! Container deployment configuration

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Container configuration (Phase 5)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ContainerConfig {
    /// Enable container support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Container runtime (docker, podman)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,

    /// Dockerfile generation configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<DockerfileConfig>,

    /// Multi-stage build configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<ContainerBuildConfig>,

    /// Registry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<RegistryConfig>,

    /// Image tags configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<ImageTagsConfig>,

    /// Container targets (for multi-image projects)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub targets: HashMap<String, ContainerTarget>,
}

/// Dockerfile generation configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct DockerfileConfig {
    /// Output path (default: Dockerfile)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    /// Base image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_image: Option<String>,

    /// Working directory in container
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,

    /// User to run as
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Exposed ports
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expose: Vec<u16>,

    /// Labels
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Additional packages to install
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,

    /// Copy instructions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub copy: Vec<CopyInstruction>,

    /// Run commands
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run: Vec<String>,

    /// Entrypoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Vec<String>>,

    /// Default command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<String>>,

    /// Healthcheck configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<ContainerHealthcheck>,

    /// Files/directories to ignore (.dockerignore)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<String>,
}

/// Copy instruction for Dockerfile
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct CopyInstruction {
    /// Source path
    pub src: String,

    /// Destination path
    pub dest: String,

    /// Owner (user:group)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chown: Option<String>,

    /// From stage (for multi-stage builds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}

/// Container healthcheck configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ContainerHealthcheck {
    /// Command to run
    pub cmd: String,

    /// Interval between checks (e.g., "30s")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,

    /// Timeout for each check (e.g., "10s")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,

    /// Start period (e.g., "5s")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_period: Option<String>,

    /// Number of retries before unhealthy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
}

/// Multi-stage build configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ContainerBuildConfig {
    /// Enable multi-stage build
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_stage: Option<bool>,

    /// Build stages
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<BuildStage>,

    /// Build arguments
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub args: HashMap<String, String>,

    /// Target stage to build
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    /// Cache configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<BuildCacheConfig>,

    /// Build context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Platform(s) to build for
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub platforms: Vec<String>,
}

/// Build stage configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct BuildStage {
    /// Stage name
    pub name: String,

    /// Base image for this stage
    pub base_image: String,

    /// Working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,

    /// Copy instructions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub copy: Vec<CopyInstruction>,

    /// Run commands
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run: Vec<String>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Build arguments used in this stage
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

/// Build cache configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct BuildCacheConfig {
    /// Enable BuildKit cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Cache type (inline, registry, local, gha)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_type: Option<String>,

    /// Cache from locations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cache_from: Vec<String>,

    /// Cache to location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_to: Option<String>,
}

/// Registry configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct RegistryConfig {
    /// Registry URL (e.g., docker.io, ghcr.io, gcr.io)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Registry username (or env var reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Registry password/token (env var reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Image name (without tag)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Push after build
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<bool>,

    /// Additional registries for multi-registry push
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mirrors: Vec<RegistryMirror>,
}

/// Registry mirror configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct RegistryMirror {
    /// Mirror registry URL
    pub url: String,

    /// Mirror image name (optional, defaults to main image name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// Image tags configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ImageTagsConfig {
    /// Tag strategy (semver, git-sha, branch, custom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,

    /// Include latest tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<bool>,

    /// Include git SHA tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_sha: Option<bool>,

    /// SHA length (default: 7)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha_length: Option<u32>,

    /// Include branch name tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<bool>,

    /// Include timestamp tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<bool>,

    /// Timestamp format (default: %Y%m%d%H%M%S)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_format: Option<String>,

    /// Custom tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<String>,

    /// Tag prefix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// Tag suffix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
}

/// Container target (for multi-image projects)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ContainerTarget {
    /// Dockerfile path (relative to project root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<String>,

    /// Build context path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Image name override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Target-specific build args
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub args: HashMap<String, String>,

    /// Target-specific tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Depends on other targets
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
}
