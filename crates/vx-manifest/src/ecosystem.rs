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
    /// Java ecosystem (java, javac, maven, gradle)
    Java,
    /// .NET ecosystem (dotnet)
    DotNet,
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
            Self::Java => write!(f, "java"),
            Self::DotNet => write!(f, "dotnet"),
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
            "java" => Ok(Self::Java),
            "dotnet" | ".net" => Ok(Self::DotNet),
            "system" => Ok(Self::System),
            _ => Err(format!("Unknown ecosystem: {}", s)),
        }
    }
}
