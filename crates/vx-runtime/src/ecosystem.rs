//! Ecosystem definitions

use serde::{Deserialize, Serialize};

/// Ecosystem represents a family of related runtimes and tools
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Ecosystem {
    /// Node.js ecosystem (node, npm, npx, yarn, pnpm, bun)
    NodeJs,
    /// Python ecosystem (uv, pip, python)
    Python,
    /// Rust ecosystem (cargo, rustc, rustup)
    Rust,
    /// Go ecosystem (go, gofmt)
    Go,
    /// System tools
    System,
    /// Custom ecosystem with a name
    Custom(String),
    /// Unknown ecosystem
    #[default]
    Unknown,
}

impl Ecosystem {
    /// Get the primary runtime for this ecosystem
    pub fn primary_runtime(&self) -> Option<&str> {
        match self {
            Ecosystem::NodeJs => Some("node"),
            Ecosystem::Python => Some("uv"),
            Ecosystem::Rust => Some("rust"),
            Ecosystem::Go => Some("go"),
            _ => None,
        }
    }

    /// Check if a runtime name belongs to this ecosystem
    pub fn contains(&self, name: &str) -> bool {
        match self {
            Ecosystem::NodeJs => {
                matches!(
                    name,
                    "node" | "nodejs" | "npm" | "npx" | "yarn" | "pnpm" | "bun"
                )
            }
            Ecosystem::Python => {
                matches!(name, "uv" | "uvx" | "python" | "pip" | "python3")
            }
            Ecosystem::Rust => {
                matches!(name, "rust" | "rustc" | "cargo" | "rustup")
            }
            Ecosystem::Go => {
                matches!(name, "go" | "golang" | "gofmt")
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ecosystem::NodeJs => write!(f, "nodejs"),
            Ecosystem::Python => write!(f, "python"),
            Ecosystem::Rust => write!(f, "rust"),
            Ecosystem::Go => write!(f, "go"),
            Ecosystem::System => write!(f, "system"),
            Ecosystem::Custom(name) => write!(f, "{}", name),
            Ecosystem::Unknown => write!(f, "unknown"),
        }
    }
}
