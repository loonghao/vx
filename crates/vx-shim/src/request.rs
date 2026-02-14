//! Package request parser for RFC 0027 implicit package execution
//!
//! Syntax: `<ecosystem>[@runtime_version]:<package>[@version][::executable]`
//!
//! Examples:
//! - `npm:typescript` - npm package typescript, default executable
//! - `npm:typescript@5.3` - npm package typescript version 5.3
//! - `npm:typescript::tsc` - npm package typescript, executable tsc
//! - `npm:typescript@5.3::tsc` - with version and executable
//! - `npm@20:typescript::tsc` - with runtime version (node 20)
//! - `pip:httpie::http` - pip package httpie, executable http
//! - `pip@3.11:ruff` - pip with Python 3.11

use crate::error::{ShimError, ShimResult};

/// Parsed package request
#[derive(Debug, Clone, PartialEq)]
pub struct PackageRequest {
    /// Package ecosystem (npm, pip, cargo, go, gem)
    pub ecosystem: String,
    /// Package name (can include scopes like @types/node)
    pub package: String,
    /// Optional package version
    pub version: Option<String>,
    /// Optional explicit executable name (default: package name)
    pub executable: Option<String>,
    /// Optional runtime version specification
    pub runtime_spec: Option<RuntimeSpec>,
}

/// Runtime version specification
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSpec {
    /// Runtime name (node, python, etc.)
    pub runtime: String,
    /// Version constraint
    pub version: String,
}

impl PackageRequest {
    /// Parse a package request string
    ///
    /// Format: `<ecosystem>[@runtime_version]:<package>[@version][::executable]`
    pub fn parse(input: &str) -> ShimResult<Self> {
        // Must contain a colon to be a package request
        let colon_pos = input.find(':').ok_or_else(|| {
            ShimError::InvalidRequest(format!("Missing ecosystem separator ':' in '{}'", input))
        })?;

        let ecosystem_part = &input[..colon_pos];
        let rest = &input[colon_pos + 1..];

        // Parse ecosystem[@runtime_version]
        let (ecosystem, runtime_spec) = Self::parse_ecosystem_part(ecosystem_part)?;

        // Parse package[@version][::executable]
        let (package, version, executable) = Self::parse_package_part(rest)?;

        Ok(Self {
            ecosystem,
            package,
            version,
            executable,
            runtime_spec,
        })
    }

    /// Check if a string looks like a package request (contains ecosystem:)
    pub fn is_package_request(input: &str) -> bool {
        // Must have ecosystem: prefix
        if let Some(colon_pos) = input.find(':') {
            let ecosystem_part = &input[..colon_pos];
            // Extract ecosystem name (before @)
            let ecosystem = ecosystem_part.split('@').next().unwrap_or("");
            // Check if it's a known ecosystem
            matches!(
                ecosystem.to_lowercase().as_str(),
                "npm" | "pip" | "cargo" | "go" | "gem" | "uv" | "bun" | "yarn" | "pnpm"
            )
        } else {
            false
        }
    }

    /// Get the executable name (explicit or default to package name)
    pub fn executable_name(&self) -> &str {
        self.executable.as_deref().unwrap_or(&self.package)
    }

    /// Parse the ecosystem part: `ecosystem[@runtime_version]`
    fn parse_ecosystem_part(part: &str) -> ShimResult<(String, Option<RuntimeSpec>)> {
        if let Some(at_pos) = part.find('@') {
            let ecosystem = part[..at_pos].to_string();
            let runtime_version = part[at_pos + 1..].to_string();

            if ecosystem.is_empty() {
                return Err(ShimError::InvalidRequest(
                    "Empty ecosystem name".to_string(),
                ));
            }
            if runtime_version.is_empty() {
                return Err(ShimError::InvalidRequest(
                    "Empty runtime version".to_string(),
                ));
            }

            // Infer runtime from ecosystem
            let runtime = Self::infer_runtime(&ecosystem)?;

            Ok((
                ecosystem,
                Some(RuntimeSpec {
                    runtime,
                    version: runtime_version,
                }),
            ))
        } else {
            if part.is_empty() {
                return Err(ShimError::InvalidRequest(
                    "Empty ecosystem name".to_string(),
                ));
            }
            Ok((part.to_string(), None))
        }
    }

    /// Parse the package part: `package[@version][::executable]`
    fn parse_package_part(part: &str) -> ShimResult<(String, Option<String>, Option<String>)> {
        // First, split by :: to get executable
        let (package_version_part, executable) = if let Some(pos) = part.find("::") {
            let exe = part[pos + 2..].to_string();
            if exe.is_empty() {
                return Err(ShimError::InvalidRequest(
                    "Empty executable name after '::'".to_string(),
                ));
            }
            (&part[..pos], Some(exe))
        } else {
            (part, None)
        };

        // Now parse package[@version]
        // Handle scoped packages like @types/node@1.0
        let (package, version) = Self::parse_package_with_version(package_version_part)?;

        if package.is_empty() {
            return Err(ShimError::InvalidRequest("Empty package name".to_string()));
        }

        Ok((package, version, executable))
    }

    /// Parse package[@version], handling scoped packages
    fn parse_package_with_version(part: &str) -> ShimResult<(String, Option<String>)> {
        if part.starts_with('@') {
            // Scoped package like @types/node or @types/node@1.0
            // Find the scope end (first /)
            if let Some(slash_pos) = part.find('/') {
                let after_slash = &part[slash_pos + 1..];
                // Look for @ in the part after the slash
                if let Some(at_pos) = after_slash.find('@') {
                    let package = format!("{}/{}", &part[..slash_pos], &after_slash[..at_pos]);
                    let version = after_slash[at_pos + 1..].to_string();
                    Ok((package, Some(version)))
                } else {
                    Ok((part.to_string(), None))
                }
            } else {
                Err(ShimError::InvalidRequest(format!(
                    "Invalid scoped package: {}",
                    part
                )))
            }
        } else {
            // Regular package like typescript or typescript@5.3
            if let Some(at_pos) = part.find('@') {
                let package = part[..at_pos].to_string();
                let version = part[at_pos + 1..].to_string();
                Ok((package, Some(version)))
            } else {
                Ok((part.to_string(), None))
            }
        }
    }

    /// Infer the runtime from ecosystem
    fn infer_runtime(ecosystem: &str) -> ShimResult<String> {
        let runtime = match ecosystem.to_lowercase().as_str() {
            "npm" | "yarn" | "pnpm" | "bun" => "node",
            "pip" | "uv" => "python",
            "cargo" => "rust",
            "go" => "go",
            "gem" => "ruby",
            _ => {
                return Err(ShimError::InvalidRequest(format!(
                    "Unknown ecosystem: {}",
                    ecosystem
                )));
            }
        };
        Ok(runtime.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_package() {
        let req = PackageRequest::parse("npm:typescript").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "typescript");
        assert_eq!(req.version, None);
        assert_eq!(req.executable, None);
        assert_eq!(req.executable_name(), "typescript");
    }

    #[test]
    fn test_package_with_version() {
        let req = PackageRequest::parse("npm:typescript@5.3").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "typescript");
        assert_eq!(req.version, Some("5.3".to_string()));
        assert_eq!(req.executable, None);
    }

    #[test]
    fn test_package_with_executable() {
        let req = PackageRequest::parse("npm:typescript::tsc").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "typescript");
        assert_eq!(req.executable, Some("tsc".to_string()));
        assert_eq!(req.executable_name(), "tsc");
    }

    #[test]
    fn test_package_with_version_and_executable() {
        let req = PackageRequest::parse("npm:typescript@5.3::tsc").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "typescript");
        assert_eq!(req.version, Some("5.3".to_string()));
        assert_eq!(req.executable, Some("tsc".to_string()));
    }

    #[test]
    fn test_package_with_runtime_version() {
        let req = PackageRequest::parse("npm@20:typescript::tsc").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "typescript");
        assert_eq!(req.executable, Some("tsc".to_string()));
        assert!(req.runtime_spec.is_some());
        let spec = req.runtime_spec.unwrap();
        assert_eq!(spec.runtime, "node");
        assert_eq!(spec.version, "20");
    }

    #[test]
    fn test_scoped_package() {
        let req = PackageRequest::parse("npm:@types/node").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "@types/node");
        assert_eq!(req.version, None);
    }

    #[test]
    fn test_scoped_package_with_version() {
        let req = PackageRequest::parse("npm:@biomejs/biome@1.5").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "@biomejs/biome");
        assert_eq!(req.version, Some("1.5".to_string()));
    }

    #[test]
    fn test_scoped_package_with_executable() {
        let req = PackageRequest::parse("npm:@biomejs/biome::biome").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "@biomejs/biome");
        assert_eq!(req.executable, Some("biome".to_string()));
    }

    #[test]
    fn test_pip_package() {
        let req = PackageRequest::parse("pip:httpie::http").unwrap();
        assert_eq!(req.ecosystem, "pip");
        assert_eq!(req.package, "httpie");
        assert_eq!(req.executable, Some("http".to_string()));
    }

    #[test]
    fn test_pip_with_python_version() {
        let req = PackageRequest::parse("pip@3.11:ruff").unwrap();
        assert_eq!(req.ecosystem, "pip");
        assert_eq!(req.package, "ruff");
        let spec = req.runtime_spec.unwrap();
        assert_eq!(spec.runtime, "python");
        assert_eq!(spec.version, "3.11");
    }

    #[test]
    fn test_is_package_request() {
        assert!(PackageRequest::is_package_request("npm:typescript"));
        assert!(PackageRequest::is_package_request("pip:ruff"));
        assert!(PackageRequest::is_package_request("cargo:ripgrep"));
        assert!(!PackageRequest::is_package_request("node"));
        assert!(!PackageRequest::is_package_request("tsc"));
        assert!(!PackageRequest::is_package_request("python@3.11"));
    }

    #[test]
    fn test_invalid_requests() {
        assert!(PackageRequest::parse("typescript").is_err()); // Missing :
        assert!(PackageRequest::parse(":typescript").is_err()); // Empty ecosystem
        assert!(PackageRequest::parse("npm:").is_err()); // Empty package
        assert!(PackageRequest::parse("npm:pkg::").is_err()); // Empty executable
    }

    // Additional tests for real-world packages

    #[test]
    fn test_openai_codex_package() {
        // npm i @openai/codex -> vx npm:@openai/codex::codex
        let req = PackageRequest::parse("npm:@openai/codex::codex").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "@openai/codex");
        assert_eq!(req.executable, Some("codex".to_string()));
        assert_eq!(req.executable_name(), "codex");
    }

    #[test]
    fn test_scoped_package_with_version_and_executable() {
        // Full syntax: ecosystem:@scope/package@version::executable
        let req = PackageRequest::parse("npm:@openai/codex@1.0.0::codex").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "@openai/codex");
        assert_eq!(req.version, Some("1.0.0".to_string()));
        assert_eq!(req.executable, Some("codex".to_string()));
    }

    #[test]
    fn test_scoped_package_with_runtime_and_executable() {
        // Full syntax with runtime: ecosystem@runtime:@scope/package::executable
        let req = PackageRequest::parse("npm@20:@openai/codex::codex").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "@openai/codex");
        assert_eq!(req.executable, Some("codex".to_string()));
        let spec = req.runtime_spec.unwrap();
        assert_eq!(spec.runtime, "node");
        assert_eq!(spec.version, "20");
    }

    #[test]
    fn test_full_syntax_scoped_package() {
        // Complete syntax: ecosystem@runtime:@scope/package@version::executable
        let req = PackageRequest::parse("npm@22:@openai/codex@1.2.3::codex").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "@openai/codex");
        assert_eq!(req.version, Some("1.2.3".to_string()));
        assert_eq!(req.executable, Some("codex".to_string()));
        let spec = req.runtime_spec.unwrap();
        assert_eq!(spec.runtime, "node");
        assert_eq!(spec.version, "22");
    }

    #[test]
    fn test_cargo_package() {
        let req = PackageRequest::parse("cargo:ripgrep::rg").unwrap();
        assert_eq!(req.ecosystem, "cargo");
        assert_eq!(req.package, "ripgrep");
        assert_eq!(req.executable, Some("rg".to_string()));
    }

    #[test]
    fn test_go_package() {
        let req = PackageRequest::parse("go:golang.org/x/tools/gopls::gopls").unwrap();
        assert_eq!(req.ecosystem, "go");
        assert_eq!(req.package, "golang.org/x/tools/gopls");
        assert_eq!(req.executable, Some("gopls".to_string()));
    }

    #[test]
    fn test_uv_ecosystem() {
        let req = PackageRequest::parse("uv:ruff::ruff").unwrap();
        assert_eq!(req.ecosystem, "uv");
        assert_eq!(req.package, "ruff");
        let req2 = PackageRequest::parse("uv@3.12:black").unwrap();
        let spec = req2.runtime_spec.unwrap();
        assert_eq!(spec.runtime, "python");
        assert_eq!(spec.version, "3.12");
    }

    #[test]
    fn test_bun_ecosystem() {
        let req = PackageRequest::parse("bun:typescript::tsc").unwrap();
        assert_eq!(req.ecosystem, "bun");
        assert_eq!(req.package, "typescript");
        assert_eq!(req.executable, Some("tsc".to_string()));
    }

    #[test]
    fn test_is_package_request_with_scoped() {
        assert!(PackageRequest::is_package_request("npm:@openai/codex"));
        assert!(PackageRequest::is_package_request(
            "npm:@openai/codex::codex"
        ));
        assert!(PackageRequest::is_package_request(
            "npm@20:@openai/codex::codex"
        ));
    }
}
