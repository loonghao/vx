//! Ecosystem definitions
//!
//! This is the canonical definition of [`Ecosystem`] used across all vx crates.
//! Other crates (vx-manifest, vx-project-analyzer, etc.) should re-export from here.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Ecosystem represents a family of related runtimes and tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Ecosystem {
    /// Node.js ecosystem (node, npm, npx, yarn, pnpm, bun)
    #[serde(alias = "node")]
    NodeJs,
    /// Python ecosystem (uv, pip, python)
    Python,
    /// Rust ecosystem (cargo, rustc, rustup)
    Rust,
    /// Go ecosystem (go, gofmt)
    Go,
    /// Git ecosystem (git) - special versioning: 2.53.0.windows.1
    Git,
    /// Ruby ecosystem (ruby, gem, bundle)
    Ruby,
    /// Java ecosystem (java, javac, jar, mvn, gradle)
    Java,
    /// .NET ecosystem (dotnet, nuget, msbuild)
    #[serde(alias = "dotnet")]
    DotNet,
    /// Development tools (cmake, ninja, task, terraform, etc.)
    DevTools,
    /// Container ecosystem (docker, kubectl, helm)
    Container,
    /// Cloud ecosystem (awscli, azcli, gcloud)
    Cloud,
    /// AI ecosystem (ollama)
    Ai,
    /// C++ ecosystem (vcpkg, cmake, meson)
    Cpp,
    /// Zig ecosystem (zig toolchain)
    Zig,
    /// System tools (not tied to a specific language)
    #[default]
    System,
    /// Custom ecosystem with a string identifier
    // NOTE: Custom(String) is intentionally omitted here because Copy requires
    // all fields to be Copy. Use System or a specific variant instead.
    // For truly custom ecosystems, use the `ecosystem` field in provider.star.
    /// Generic/standalone runtimes
    Generic,
    /// Unknown ecosystem
    Unknown,
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeJs => write!(f, "nodejs"),
            Self::Python => write!(f, "python"),
            Self::Rust => write!(f, "rust"),
            Self::Go => write!(f, "go"),
            Self::Git => write!(f, "git"),
            Self::Ruby => write!(f, "ruby"),
            Self::Java => write!(f, "java"),
            Self::DotNet => write!(f, "dotnet"),
            Self::DevTools => write!(f, "devtools"),
            Self::Container => write!(f, "container"),
            Self::Cloud => write!(f, "cloud"),
            Self::Ai => write!(f, "ai"),
            Self::Cpp => write!(f, "cpp"),
            Self::Zig => write!(f, "zig"),
            Self::System => write!(f, "system"),
            Self::Generic => write!(f, "generic"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for Ecosystem {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "nodejs" | "node" => Ok(Self::NodeJs),
            "python" => Ok(Self::Python),
            "rust" => Ok(Self::Rust),
            "go" | "golang" => Ok(Self::Go),
            "git" => Ok(Self::Git),
            "ruby" | "gem" => Ok(Self::Ruby),
            "java" => Ok(Self::Java),
            "dotnet" | ".net" => Ok(Self::DotNet),
            "devtools" => Ok(Self::DevTools),
            "container" => Ok(Self::Container),
            "cloud" => Ok(Self::Cloud),
            "ai" => Ok(Self::Ai),
            "cpp" | "c++" => Ok(Self::Cpp),
            "zig" => Ok(Self::Zig),
            "system" => Ok(Self::System),
            "generic" => Ok(Self::Generic),
            "unknown" => Ok(Self::Unknown),
            // Fallback: unknown ecosystems map to System
            _ => Ok(Self::System),
        }
    }
}

impl Ecosystem {
    /// Get the primary runtime for this ecosystem
    pub fn primary_runtime(&self) -> Option<&'static str> {
        match self {
            Self::NodeJs => Some("node"),
            Self::Python => Some("uv"),
            Self::Rust => Some("rust"),
            Self::Go => Some("go"),
            Self::Git => Some("git"),
            Self::Ruby => Some("ruby"),
            Self::Java => Some("java"),
            Self::DotNet => Some("dotnet"),
            Self::Zig => Some("zig"),
            _ => None,
        }
    }

    /// Get the package manager directory name for global packages
    ///
    /// Returns the directory name used in `~/.vx/packages/{dir}/`
    pub fn package_manager_dir(&self) -> Option<&'static str> {
        match self {
            Self::NodeJs => Some("npm"),
            Self::Python => Some("pip"),
            Self::Rust => Some("cargo"),
            Self::Go => Some("go"),
            Self::Ruby => Some("gem"),
            _ => None,
        }
    }

    /// Check if this ecosystem supports global package installation
    pub fn supports_global_packages(&self) -> bool {
        matches!(
            self,
            Self::NodeJs | Self::Python | Self::Rust | Self::Go | Self::Ruby
        )
    }

    /// Check if a runtime name belongs to this ecosystem
    pub fn contains(&self, name: &str) -> bool {
        match self {
            Self::NodeJs => {
                matches!(
                    name,
                    "node" | "nodejs" | "npm" | "npx" | "yarn" | "pnpm" | "bun"
                )
            }
            Self::Python => {
                matches!(name, "uv" | "uvx" | "python" | "pip" | "python3")
            }
            Self::Rust => {
                matches!(name, "rust" | "rustc" | "cargo" | "rustup")
            }
            Self::Go => {
                matches!(name, "go" | "golang" | "gofmt")
            }
            Self::Git => {
                matches!(name, "git")
            }
            Self::Ruby => {
                matches!(name, "ruby" | "gem" | "bundle" | "bundler")
            }
            Self::Java => {
                matches!(name, "java" | "javac" | "jar" | "mvn" | "gradle")
            }
            Self::DotNet => {
                matches!(name, "dotnet" | "dotnet-sdk" | "nuget" | "msbuild")
            }
            Self::Zig => {
                matches!(name, "zig")
            }
            _ => false,
        }
    }

    /// Parse ecosystem from package manager name (npm, pip, cargo, go, gem)
    pub fn from_package_manager(pm: &str) -> Option<Self> {
        match pm.to_lowercase().as_str() {
            "npm" | "yarn" | "pnpm" => Some(Self::NodeJs),
            "pip" | "uv" | "uvx" => Some(Self::Python),
            "cargo" => Some(Self::Rust),
            "go" => Some(Self::Go),
            "gem" | "bundle" => Some(Self::Ruby),
            _ => None,
        }
    }
}
