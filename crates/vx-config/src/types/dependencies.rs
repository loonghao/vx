//! Dependencies configuration

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dependencies configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DependenciesConfig {
    /// Generate lockfile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockfile: Option<bool>,

    /// Run audit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit: Option<bool>,

    /// Auto-update strategy (none, patch, minor, major)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update: Option<String>,

    /// Node.js dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeDependenciesConfig>,

    /// Python dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python: Option<PythonDependenciesConfig>,

    /// Go dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub go: Option<GoDependenciesConfig>,

    /// C++ dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpp: Option<CppDependenciesConfig>,

    /// Dependency constraints
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub constraints: HashMap<String, ConstraintValue>,
}

/// Node.js dependencies configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct NodeDependenciesConfig {
    /// Package manager (npm, yarn, pnpm, bun)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,

    /// Registry URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
}

/// Python dependencies configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PythonDependenciesConfig {
    /// Index URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_url: Option<String>,

    /// Extra index URLs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_index_urls: Vec<String>,
}

/// Go dependencies configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct GoDependenciesConfig {
    /// Go proxy URL (e.g., https://goproxy.cn, https://proxy.golang.org)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,

    /// Go private modules (comma-separated patterns)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<String>,

    /// Go sum database URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sumdb: Option<String>,

    /// Disable Go sum database
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nosumdb: Option<String>,

    /// Go vendor mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<bool>,

    /// Go module download mode (readonly, vendor, mod)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_mode: Option<String>,
}

/// C++ dependencies configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct CppDependenciesConfig {
    /// Package manager (conan, vcpkg, cmake)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,

    /// Conan remote URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conan_remote: Option<String>,

    /// vcpkg root path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcpkg_root: Option<String>,

    /// vcpkg triplet (e.g., x64-windows, x64-linux, x64-osx)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcpkg_triplet: Option<String>,

    /// CMake generator (Ninja, "Unix Makefiles", "Visual Studio 17 2022")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmake_generator: Option<String>,

    /// CMake build type (Debug, Release, RelWithDebInfo, MinSizeRel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmake_build_type: Option<String>,

    /// Additional CMake options
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub cmake_options: HashMap<String, String>,

    /// C++ standard (11, 14, 17, 20, 23)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub std: Option<String>,

    /// Compiler (gcc, clang, msvc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
}

/// Constraint value
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ConstraintValue {
    /// Version constraint
    Version(String),
    /// Detailed constraint
    Detailed(ConstraintDetails),
}

/// Detailed constraint
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ConstraintDetails {
    /// Allowed licenses
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub licenses: Vec<String>,
}
