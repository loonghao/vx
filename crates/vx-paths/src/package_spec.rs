//! Package specification parsing (RFC 0025)
//!
//! This module provides utilities for parsing package specifications in various formats:
//! - `npm:typescript@5.3`
//! - `pip:black@24.1`
//! - `cargo:ripgrep@14`
//! - `go:golangci-lint@1.55`
//! - `gem:bundler@2.5`
//! - `typescript@5.3` (auto-detect ecosystem)

use anyhow::{anyhow, Result};
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
    fn parse_package_version(s: &str) -> Result<(String, Option<String>)> {
        if let Some(at_pos) = s.rfind('@') {
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
            if s.is_empty() {
                return Err(anyhow!("Empty package name"));
            }
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
            "typescript",
            "tsc",
            "eslint",
            "prettier",
            "webpack",
            "vite",
            "react",
            "vue",
            "angular",
            "next",
            "nuxt",
            "nx",
            "turbo",
            "jest",
            "vitest",
            "mocha",
            "cypress",
            "playwright",
            "nodemon",
            "ts-node",
            "tsx",
            "esbuild",
            "rollup",
            "parcel",
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
            package, package, package
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
}
