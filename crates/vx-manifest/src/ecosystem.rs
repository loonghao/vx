//! Ecosystem definitions

use serde::{Deserialize, Serialize};
use std::fmt;

/// Ecosystem that a Provider belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Ecosystem {
    /// Node.js ecosystem (node, npm, yarn, pnpm, bun)
    #[serde(alias = "node")]
    NodeJs,
    /// Python ecosystem (python, pip, uv, uvx)
    Python,
    /// Rust ecosystem (cargo, rustc, rustup)
    Rust,
    /// Go ecosystem (go, gofmt)
    Go,
    /// Ruby ecosystem (ruby, gem, bundle) - RFC 0025
    Ruby,
    /// Java ecosystem (java, javac, maven, gradle)
    Java,
    /// .NET ecosystem (dotnet)
    DotNet,
    /// Development tools (git, cmake, ninja, task, terraform, etc.)
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
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeJs => write!(f, "nodejs"),
            Self::Python => write!(f, "python"),
            Self::Rust => write!(f, "rust"),
            Self::Go => write!(f, "go"),
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
            _ => Err(format!("Unknown ecosystem: {}", s)),
        }
    }
}

impl Ecosystem {
    /// Get the package manager directory name for global packages (RFC 0025)
    ///
    /// Returns the directory name used in ~/.vx/packages/{dir}/
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

    /// Get the default runtime name for this ecosystem
    pub fn default_runtime(&self) -> Option<&'static str> {
        match self {
            Self::NodeJs => Some("node"),
            Self::Python => Some("python"),
            Self::Rust => Some("rust"),
            Self::Go => Some("go"),
            Self::Ruby => Some("ruby"),
            Self::Java => Some("java"),
            Self::DotNet => Some("dotnet"),
            Self::Zig => Some("zig"),
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
