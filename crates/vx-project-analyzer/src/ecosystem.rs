//! Ecosystem definitions

use serde::{Deserialize, Serialize};

/// Supported ecosystems/languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ecosystem {
    /// Python ecosystem (uv, pip, poetry, etc.)
    Python,
    /// Node.js ecosystem (npm, yarn, pnpm, bun)
    NodeJs,
    /// Rust ecosystem (cargo, rustup)
    Rust,
    /// Go ecosystem
    Go,
    /// Java ecosystem (maven, gradle)
    Java,
    /// Unknown/Other
    Unknown,
}

impl Ecosystem {
    /// Get display name for the ecosystem
    pub fn display_name(&self) -> &'static str {
        match self {
            Ecosystem::Python => "Python",
            Ecosystem::NodeJs => "Node.js",
            Ecosystem::Rust => "Rust",
            Ecosystem::Go => "Go",
            Ecosystem::Java => "Java",
            Ecosystem::Unknown => "Unknown",
        }
    }

    /// Get common file indicators for this ecosystem
    pub fn indicator_files(&self) -> &'static [&'static str] {
        match self {
            Ecosystem::Python => &[
                "pyproject.toml",
                "setup.py",
                "requirements.txt",
                "Pipfile",
                "uv.lock",
                "poetry.lock",
            ],
            Ecosystem::NodeJs => &[
                "package.json",
                "package-lock.json",
                "yarn.lock",
                "pnpm-lock.yaml",
                "bun.lockb",
            ],
            Ecosystem::Rust => &["Cargo.toml", "Cargo.lock"],
            Ecosystem::Go => &["go.mod", "go.sum"],
            Ecosystem::Java => &["pom.xml", "build.gradle", "build.gradle.kts"],
            Ecosystem::Unknown => &[],
        }
    }
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
