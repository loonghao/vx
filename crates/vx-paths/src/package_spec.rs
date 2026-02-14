//! Package specification parsing (RFC 0025)
//!
//! This module provides utilities for parsing package specifications in various formats:
//! - `npm:typescript@5.3`
//! - `pip:black@24.1`
//! - `cargo:ripgrep@14`
//! - `go:golangci-lint@1.55`
//! - `gem:bundler@2.5`
//! - `typescript@5.3` (auto-detect ecosystem)

use anyhow::{Result, anyhow};
use std::fmt;

/// Parsed package specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSpec {
    /// Package manager / ecosystem (npm, pip, cargo, go, gem)
    pub ecosystem: String,
    /// Package name
    pub package: String,
    /// Version (optional, "latest" if not specified)
    pub version: Option<String>,
}

impl PackageSpec {
    /// Create a new PackageSpec
    pub fn new(ecosystem: impl Into<String>, package: impl Into<String>) -> Self {
        Self {
            ecosystem: ecosystem.into(),
            package: package.into(),
            version: None,
        }
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Get version or "latest"
    pub fn version_or_latest(&self) -> &str {
        self.version.as_deref().unwrap_or("latest")
    }

    /// Parse a package specification string
    ///
    /// Supported formats:
    /// - `ecosystem:package@version` (e.g., `npm:typescript@5.3`)
    /// - `ecosystem:package` (e.g., `pip:black`)
    /// - `package@version` (auto-detect, e.g., `typescript@5.3`)
    /// - `package` (auto-detect, e.g., `typescript`)
    pub fn parse(spec: &str) -> Result<Self> {
        let spec = spec.trim();

        if spec.is_empty() {
            return Err(anyhow!("Empty package specification"));
        }

        // Check if ecosystem is specified (contains :)
        if let Some(colon_pos) = spec.find(':') {
            let ecosystem = &spec[..colon_pos];
            let rest = &spec[colon_pos + 1..];

            if ecosystem.is_empty() {
                return Err(anyhow!("Empty ecosystem in specification: {}", spec));
            }
            if rest.is_empty() {
                return Err(anyhow!("Empty package name in specification: {}", spec));
            }

            // Validate ecosystem
            Self::validate_ecosystem(ecosystem)?;

            // Parse package@version
            let (package, version) = Self::parse_package_version(rest)?;

            return Ok(Self {
                ecosystem: ecosystem.to_lowercase(),
                package,
                version,
            });
        }

        // No ecosystem specified, try to auto-detect
        let (package, version) = Self::parse_package_version(spec)?;
        let ecosystem = Self::detect_ecosystem(&package)?;

        Ok(Self {
            ecosystem,
            package,
            version,
        })
    }

    /// Parse package@version format
    ///
    /// Handles scoped npm packages like @scope/package@version correctly
    fn parse_package_version(s: &str) -> Result<(String, Option<String>)> {
        if s.is_empty() {
            return Err(anyhow!("Empty package name"));
        }

        // Handle scoped npm packages (e.g., @scope/package@version)
        // For scoped packages, we need to find @ that comes after /
        let version_at_pos = if s.starts_with('@') {
            // Scoped package - find @ after the first /
            if let Some(slash_pos) = s.find('/') {
                // Look for @ after the slash
                s[slash_pos..].rfind('@').map(|pos| slash_pos + pos)
            } else {
                // No slash found, treat as regular package
                s.rfind('@')
            }
        } else {
            // Regular package
            s.rfind('@')
        };

        if let Some(at_pos) = version_at_pos {
            // Make sure we're not just finding the @ at the start of a scoped package
            if at_pos == 0 {
                // This is a scoped package without version (@scope/package)
                return Ok((s.to_string(), None));
            }

            let package = &s[..at_pos];
            let version = &s[at_pos + 1..];

            if package.is_empty() {
                return Err(anyhow!("Empty package name"));
            }
            if version.is_empty() {
                return Err(anyhow!("Empty version after @"));
            }

            Ok((package.to_string(), Some(version.to_string())))
        } else {
            Ok((s.to_string(), None))
        }
    }

    /// Validate ecosystem name
    fn validate_ecosystem(ecosystem: &str) -> Result<()> {
        let valid = matches!(
            ecosystem.to_lowercase().as_str(),
            "npm" | "pip" | "cargo" | "go" | "gem" | "yarn" | "pnpm" | "uv" | "uvx"
        );

        if valid {
            Ok(())
        } else {
            Err(anyhow!(
                "Unknown ecosystem '{}'. Valid options: npm, pip, cargo, go, gem",
                ecosystem
            ))
        }
    }

    /// Detect ecosystem from package name using common package registry
    fn detect_ecosystem(package: &str) -> Result<String> {
        // Common npm packages
        let npm_packages = [
            // Build tools
            "typescript",
            "tsc",
            "esbuild",
            "rollup",
            "parcel",
            "webpack",
            "vite",
            "turbo",
            "nx",
            // Frameworks
            "react",
            "vue",
            "angular",
            "next",
            "nuxt",
            "svelte",
            "astro",
            "remix",
            // Testing
            "jest",
            "vitest",
            "mocha",
            "cypress",
            "playwright",
            // Linting & Formatting
            "eslint",
            "prettier",
            "biome",
            // Runtime tools
            "nodemon",
            "ts-node",
            "tsx",
            // Video/Media
            "remotion",
            "ffmpeg-static",
            // AI/CLI tools
            "@anthropic-ai/claude-code",
            "claude-code",
            "@openai/codex",
            "codex",
            // Package managers & tools
            "npm",
            "yarn",
            "pnpm",
            "bun",
            // Database tools
            "prisma",
            "drizzle-kit",
            // API tools
            "openapi-typescript",
            "swagger-cli",
            // Other popular tools
            "zx",
            "concurrently",
            "npm-run-all",
            "cross-env",
            "dotenv-cli",
            "http-server",
            "serve",
            "create-react-app",
            "create-next-app",
            "create-vite",
            "@biomejs/biome",
        ];

        // Common pip packages
        let pip_packages = [
            "black",
            "ruff",
            "mypy",
            "pytest",
            "nox",
            "tox",
            "pre-commit",
            "flask",
            "django",
            "fastapi",
            "uvicorn",
            "gunicorn",
            "poetry",
            "pdm",
            "hatch",
            "flit",
            "pipx",
            "jupyter",
            "notebook",
            "ipython",
        ];

        // Common cargo packages
        let cargo_packages = [
            "ripgrep",
            "rg",
            "fd-find",
            "fd",
            "bat",
            "exa",
            "eza",
            "tokei",
            "hyperfine",
            "just",
            "cargo-watch",
            "cargo-edit",
            "cross",
            "wasm-pack",
            "trunk",
            "tauri-cli",
        ];

        // Common go packages
        let go_packages = [
            "golangci-lint",
            "gofumpt",
            "staticcheck",
            "dlv",
            "gopls",
            "cobra-cli",
            "mockgen",
            "wire",
        ];

        // Common gem packages
        let gem_packages = [
            "bundler",
            "rails",
            "rake",
            "rspec",
            "rubocop",
            "pry",
            "solargraph",
            "jekyll",
            "sass",
            "cocoapods",
        ];

        let package_lower = package.to_lowercase();

        if npm_packages.iter().any(|p| *p == package_lower) {
            return Ok("npm".to_string());
        }
        if pip_packages.iter().any(|p| *p == package_lower) {
            return Ok("pip".to_string());
        }
        if cargo_packages.iter().any(|p| *p == package_lower) {
            return Ok("cargo".to_string());
        }
        if go_packages.iter().any(|p| *p == package_lower) {
            return Ok("go".to_string());
        }
        if gem_packages.iter().any(|p| *p == package_lower) {
            return Ok("gem".to_string());
        }

        // Default to npm for unknown packages (most common case)
        // In practice, user should specify ecosystem explicitly
        Err(anyhow!(
            "Cannot auto-detect ecosystem for '{}'. Please specify explicitly (e.g., npm:{} or pip:{})",
            package,
            package,
            package
        ))
    }

    /// Normalize ecosystem name to standard form
    pub fn normalize_ecosystem(ecosystem: &str) -> String {
        match ecosystem.to_lowercase().as_str() {
            "npm" | "yarn" | "pnpm" => "npm".to_string(),
            "pip" | "uv" | "uvx" => "pip".to_string(),
            "cargo" => "cargo".to_string(),
            "go" | "golang" => "go".to_string(),
            "gem" | "ruby" | "bundle" => "gem".to_string(),
            other => other.to_string(),
        }
    }
}

impl fmt::Display for PackageSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref version) = self.version {
            write!(f, "{}:{}@{}", self.ecosystem, self.package, version)
        } else {
            write!(f, "{}:{}", self.ecosystem, self.package)
        }
    }
}

impl std::str::FromStr for PackageSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_spec() {
        let spec = PackageSpec::parse("npm:typescript@5.3").unwrap();
        assert_eq!(spec.ecosystem, "npm");
        assert_eq!(spec.package, "typescript");
        assert_eq!(spec.version, Some("5.3".to_string()));
    }

    #[test]
    fn test_parse_without_version() {
        let spec = PackageSpec::parse("pip:black").unwrap();
        assert_eq!(spec.ecosystem, "pip");
        assert_eq!(spec.package, "black");
        assert_eq!(spec.version, None);
        assert_eq!(spec.version_or_latest(), "latest");
    }

    #[test]
    fn test_parse_auto_detect_npm() {
        let spec = PackageSpec::parse("typescript@5.3").unwrap();
        assert_eq!(spec.ecosystem, "npm");
        assert_eq!(spec.package, "typescript");
        assert_eq!(spec.version, Some("5.3".to_string()));
    }

    #[test]
    fn test_parse_auto_detect_pip() {
        let spec = PackageSpec::parse("black@24.1").unwrap();
        assert_eq!(spec.ecosystem, "pip");
        assert_eq!(spec.package, "black");
    }

    #[test]
    fn test_parse_auto_detect_cargo() {
        let spec = PackageSpec::parse("ripgrep@14").unwrap();
        assert_eq!(spec.ecosystem, "cargo");
        assert_eq!(spec.package, "ripgrep");
    }

    #[test]
    fn test_parse_unknown_package() {
        let result = PackageSpec::parse("unknown-package@1.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_ecosystem() {
        let result = PackageSpec::parse("invalid:package@1.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_spec() {
        let result = PackageSpec::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        let spec = PackageSpec::new("npm", "typescript").with_version("5.3");
        assert_eq!(spec.to_string(), "npm:typescript@5.3");

        let spec_no_version = PackageSpec::new("pip", "black");
        assert_eq!(spec_no_version.to_string(), "pip:black");
    }

    #[test]
    fn test_normalize_ecosystem() {
        assert_eq!(PackageSpec::normalize_ecosystem("yarn"), "npm");
        assert_eq!(PackageSpec::normalize_ecosystem("pnpm"), "npm");
        assert_eq!(PackageSpec::normalize_ecosystem("uv"), "pip");
        assert_eq!(PackageSpec::normalize_ecosystem("bundle"), "gem");
    }

    #[test]
    fn test_from_str() {
        let spec: PackageSpec = "cargo:ripgrep@14".parse().unwrap();
        assert_eq!(spec.ecosystem, "cargo");
        assert_eq!(spec.package, "ripgrep");
    }

    // Real-world package tests for global package management (RFC 0025)

    #[test]
    fn test_parse_vitest() {
        // vitest is a popular npm test runner
        let spec = PackageSpec::parse("vitest@1.0.0").unwrap();
        assert_eq!(spec.ecosystem, "npm");
        assert_eq!(spec.package, "vitest");
        assert_eq!(spec.version, Some("1.0.0".to_string()));

        // Also test explicit ecosystem
        let spec2 = PackageSpec::parse("npm:vitest@2.0").unwrap();
        assert_eq!(spec2.ecosystem, "npm");
        assert_eq!(spec2.package, "vitest");
    }

    #[test]
    fn test_parse_remotion() {
        // remotion is a video creation framework for React
        let spec = PackageSpec::parse("remotion@4.0").unwrap();
        assert_eq!(spec.ecosystem, "npm");
        assert_eq!(spec.package, "remotion");
        assert_eq!(spec.version, Some("4.0".to_string()));

        // Without version
        let spec2 = PackageSpec::parse("npm:remotion").unwrap();
        assert_eq!(spec2.ecosystem, "npm");
        assert_eq!(spec2.package, "remotion");
        assert_eq!(spec2.version, None);
    }

    #[test]
    fn test_parse_claude_code() {
        // @anthropic-ai/claude-code - AI coding assistant CLI
        let spec = PackageSpec::parse("npm:@anthropic-ai/claude-code@1.0").unwrap();
        assert_eq!(spec.ecosystem, "npm");
        assert_eq!(spec.package, "@anthropic-ai/claude-code");
        assert_eq!(spec.version, Some("1.0".to_string()));

        // Short form (auto-detect)
        let spec2 = PackageSpec::parse("claude-code@0.2.0").unwrap();
        assert_eq!(spec2.ecosystem, "npm");
        assert_eq!(spec2.package, "claude-code");
    }

    #[test]
    fn test_parse_codex() {
        // @openai/codex - OpenAI Codex CLI
        let spec = PackageSpec::parse("npm:@openai/codex@1.0").unwrap();
        assert_eq!(spec.ecosystem, "npm");
        assert_eq!(spec.package, "@openai/codex");
        assert_eq!(spec.version, Some("1.0".to_string()));

        // Short form
        let spec2 = PackageSpec::parse("codex").unwrap();
        assert_eq!(spec2.ecosystem, "npm");
        assert_eq!(spec2.package, "codex");
    }

    #[test]
    fn test_parse_common_npm_tools() {
        // Build tools
        let vite = PackageSpec::parse("vite@5.0").unwrap();
        assert_eq!(vite.ecosystem, "npm");
        assert_eq!(vite.package, "vite");

        let turbo = PackageSpec::parse("turbo").unwrap();
        assert_eq!(turbo.ecosystem, "npm");
        assert_eq!(turbo.package, "turbo");

        let esbuild = PackageSpec::parse("esbuild@0.20").unwrap();
        assert_eq!(esbuild.ecosystem, "npm");

        // Frameworks
        let next = PackageSpec::parse("next@14").unwrap();
        assert_eq!(next.ecosystem, "npm");

        let nuxt = PackageSpec::parse("nuxt@3.10").unwrap();
        assert_eq!(nuxt.ecosystem, "npm");

        // Testing
        let playwright = PackageSpec::parse("playwright@1.42").unwrap();
        assert_eq!(playwright.ecosystem, "npm");

        let cypress = PackageSpec::parse("cypress@13").unwrap();
        assert_eq!(cypress.ecosystem, "npm");
    }

    #[test]
    fn test_parse_common_pip_tools() {
        // Python linters
        let ruff = PackageSpec::parse("ruff@0.3").unwrap();
        assert_eq!(ruff.ecosystem, "pip");
        assert_eq!(ruff.package, "ruff");

        let black = PackageSpec::parse("black@24.2").unwrap();
        assert_eq!(black.ecosystem, "pip");

        let mypy = PackageSpec::parse("mypy@1.8").unwrap();
        assert_eq!(mypy.ecosystem, "pip");

        // Frameworks
        let fastapi = PackageSpec::parse("fastapi@0.110").unwrap();
        assert_eq!(fastapi.ecosystem, "pip");

        let django = PackageSpec::parse("django@5.0").unwrap();
        assert_eq!(django.ecosystem, "pip");

        // Testing
        let pytest = PackageSpec::parse("pytest@8.0").unwrap();
        assert_eq!(pytest.ecosystem, "pip");

        let nox = PackageSpec::parse("nox").unwrap();
        assert_eq!(nox.ecosystem, "pip");
    }

    #[test]
    fn test_parse_common_cargo_tools() {
        // CLI tools
        let ripgrep = PackageSpec::parse("ripgrep@14").unwrap();
        assert_eq!(ripgrep.ecosystem, "cargo");
        assert_eq!(ripgrep.package, "ripgrep");

        let fd = PackageSpec::parse("fd-find@9").unwrap();
        assert_eq!(fd.ecosystem, "cargo");

        let bat = PackageSpec::parse("bat@0.24").unwrap();
        assert_eq!(bat.ecosystem, "cargo");

        let hyperfine = PackageSpec::parse("hyperfine@1.18").unwrap();
        assert_eq!(hyperfine.ecosystem, "cargo");

        let just = PackageSpec::parse("just@1.24").unwrap();
        assert_eq!(just.ecosystem, "cargo");

        // Cargo extensions
        let tauri = PackageSpec::parse("tauri-cli@2.0").unwrap();
        assert_eq!(tauri.ecosystem, "cargo");
    }

    #[test]
    fn test_parse_common_go_tools() {
        let golangci = PackageSpec::parse("golangci-lint@1.56").unwrap();
        assert_eq!(golangci.ecosystem, "go");
        assert_eq!(golangci.package, "golangci-lint");

        let gopls = PackageSpec::parse("gopls").unwrap();
        assert_eq!(gopls.ecosystem, "go");
    }

    #[test]
    fn test_parse_common_gem_tools() {
        let bundler = PackageSpec::parse("bundler@2.5").unwrap();
        assert_eq!(bundler.ecosystem, "gem");
        assert_eq!(bundler.package, "bundler");

        let rails = PackageSpec::parse("rails@7.1").unwrap();
        assert_eq!(rails.ecosystem, "gem");

        let rubocop = PackageSpec::parse("rubocop@1.60").unwrap();
        assert_eq!(rubocop.ecosystem, "gem");
    }

    #[test]
    fn test_parse_scoped_npm_packages() {
        // Scoped packages like @org/package
        let biome = PackageSpec::parse("npm:@biomejs/biome@1.5").unwrap();
        assert_eq!(biome.ecosystem, "npm");
        assert_eq!(biome.package, "@biomejs/biome");
        assert_eq!(biome.version, Some("1.5".to_string()));

        let claude = PackageSpec::parse("npm:@anthropic-ai/claude-code").unwrap();
        assert_eq!(claude.ecosystem, "npm");
        assert_eq!(claude.package, "@anthropic-ai/claude-code");
        assert_eq!(claude.version, None);
    }

    #[test]
    fn test_version_constraints() {
        // Simple version
        let spec1 = PackageSpec::parse("npm:typescript@5.3.3").unwrap();
        assert_eq!(spec1.version, Some("5.3.3".to_string()));

        // Semver range (passed as string)
        let spec2 = PackageSpec::parse("npm:typescript@^5.0").unwrap();
        assert_eq!(spec2.version, Some("^5.0".to_string()));

        // Latest
        let spec3 = PackageSpec::parse("npm:typescript").unwrap();
        assert_eq!(spec3.version, None);
        assert_eq!(spec3.version_or_latest(), "latest");
    }
}
