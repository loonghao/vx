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
    /// C++ ecosystem (cmake, meson, make)
    Cpp,
    /// .NET/C# ecosystem (dotnet, nuget, msbuild)
    DotNet,
    /// Java ecosystem (maven, gradle)
    Java,
    /// Bun runtime ecosystem
    Bun,
    /// Deno runtime ecosystem
    Deno,
    /// Nix package manager ecosystem
    Nix,
    /// Zig programming language ecosystem
    Zig,
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
            Ecosystem::Cpp => "C++",
            Ecosystem::DotNet => ".NET/C#",
            Ecosystem::Java => "Java",
            Ecosystem::Bun => "Bun",
            Ecosystem::Deno => "Deno",
            Ecosystem::Nix => "Nix",
            Ecosystem::Zig => "Zig",
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
            Ecosystem::Cpp => &["CMakeLists.txt", "meson.build", "Makefile"],
            Ecosystem::DotNet => &[
                "*.csproj",
                "*.sln",
                "*.fsproj",
                "global.json",
                "Directory.Build.props",
            ],
            Ecosystem::Bun => &["bun.lockb", "bunfig.toml", "bunfig.toml5"],
            Ecosystem::Deno => &["deno.json", "deno.jsonc", "deno.lock"],
            Ecosystem::Nix => &["flake.nix", "default.nix", "shell.nix"],
            Ecosystem::Zig => &["build.zig", "zig.mod"],
            Ecosystem::Java => &["pom.xml", "build.gradle", "build.gradle.kts"],
            Ecosystem::Bun => &["bun.lockb", "bunfig.toml"],
            Ecosystem::Deno => &["deno.json", "deno.lock"],
            Ecosystem::Nix => &["flake.nix", "shell.nix"],
            Ecosystem::Zig => &["build.zig"],
            Ecosystem::Unknown => &[],
        }
    }
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
