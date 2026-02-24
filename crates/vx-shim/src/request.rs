//! Package request parser for RFC 0027 implicit package execution
//!
//! Syntax: `<ecosystem>[@runtime_version]:<package>[@version][::executable_or_shell]`
//!
//! Examples:
//! - `npm:typescript` - npm package typescript, default executable
//! - `npm:typescript@5.3` - npm package typescript version 5.3
//! - `npm:typescript::tsc` - npm package typescript, executable tsc
//! - `npm:typescript@5.3::tsc` - with version and executable
//! - `npm@20:typescript::tsc` - with runtime version (node 20)
//! - `pip:httpie::http` - pip package httpie, executable http
//! - `pip@3.11:ruff` - pip with Python 3.11
//! - `npm:codex::cmd` - npm package codex, launch cmd shell with package environment
//! - `npm:codex::powershell` - npm package codex, launch powershell with package environment

use crate::error::{ShimError, ShimResult};

/// Parsed package part: (package, version, executable, shell)
type ParsedPackagePart = (String, Option<String>, Option<String>, Option<String>);

/// Known shell executables that can be launched with package environment
const KNOWN_SHELLS: &[&str] = &[
    "cmd",
    "powershell",
    "pwsh",
    "bash",
    "sh",
    "zsh",
    "fish",
    "dash",
    "ksh",
    "csh",
    "tcsh",
];

/// Check if a name is a known shell executable
fn is_known_shell(name: &str) -> bool {
    KNOWN_SHELLS.contains(&name.to_lowercase().as_str())
}

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
    /// This is None when `shell` is set.
    pub executable: Option<String>,
    /// Optional shell to launch with package environment
    /// When set, instead of running an executable, we launch a shell
    /// with the package's runtime environment configured.
    pub shell: Option<String>,
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
    /// Format: `<ecosystem>[@runtime_version]:<package>[@version][::executable_or_shell]`
    pub fn parse(input: &str) -> ShimResult<Self> {
        // Must contain a colon to be a package request
        let colon_pos = input.find(':').ok_or_else(|| {
            ShimError::InvalidRequest(format!("Missing ecosystem separator ':' in '{}'", input))
        })?;

        let ecosystem_part = &input[..colon_pos];
        let rest = &input[colon_pos + 1..];

        // Parse ecosystem[@runtime_version]
        let (ecosystem, runtime_spec) = Self::parse_ecosystem_part(ecosystem_part)?;

        // Parse package[@version][::executable_or_shell]
        let (package, version, executable, shell) = Self::parse_package_part(rest)?;

        Ok(Self {
            ecosystem,
            package,
            version,
            executable,
            shell,
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
                "npm" | "pip" | "cargo" | "go" | "gem" | "uv" | "uvx" | "bun" | "yarn" | "pnpm"
            )
        } else {
            false
        }
    }

    /// Get the executable name (explicit or default to package name)
    /// Returns None if this is a shell request (check `is_shell_request()` first)
    pub fn executable_name(&self) -> &str {
        self.executable.as_deref().unwrap_or(&self.package)
    }

    /// Check if this request wants to launch a shell with package environment
    pub fn is_shell_request(&self) -> bool {
        self.shell.is_some()
    }

    /// Get the shell name if this is a shell request
    pub fn shell_name(&self) -> Option<&str> {
        self.shell.as_deref()
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

    /// Parse the package part: `package[@version][::executable_or_shell]`
    fn parse_package_part(part: &str) -> ShimResult<ParsedPackagePart> {
        // First, split by :: to get executable or shell
        let (package_version_part, executable_or_shell) = if let Some(pos) = part.find("::") {
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

        // Determine if the :: part is a shell or an executable
        let (executable, shell) = if let Some(exe_or_shell) = executable_or_shell {
            if is_known_shell(&exe_or_shell) {
                // It's a shell - we'll launch this shell with package environment
                (None, Some(exe_or_shell))
            } else {
                // It's an executable
                (Some(exe_or_shell), None)
            }
        } else {
            (None, None)
        };

        Ok((package, version, executable, shell))
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
            "pip" | "uv" | "uvx" => "python",
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

    // Tests for shell syntax (ecosystem:package::shell)

    #[test]
    fn test_package_with_cmd_shell() {
        // npm:codex::cmd - launch cmd shell with codex environment
        let req = PackageRequest::parse("npm:codex::cmd").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "codex");
        assert_eq!(req.executable, None);
        assert_eq!(req.shell, Some("cmd".to_string()));
        assert!(req.is_shell_request());
        assert_eq!(req.shell_name(), Some("cmd"));
    }

    #[test]
    fn test_package_with_powershell_shell() {
        // npm:codex::powershell - launch powershell with codex environment
        let req = PackageRequest::parse("npm:codex::powershell").unwrap();
        assert_eq!(req.ecosystem, "npm");
        assert_eq!(req.package, "codex");
        assert_eq!(req.shell, Some("powershell".to_string()));
        assert!(req.is_shell_request());
    }

    #[test]
    fn test_package_with_bash_shell() {
        // pip:httpie::bash - launch bash with httpie environment
        let req = PackageRequest::parse("pip:httpie::bash").unwrap();
        assert_eq!(req.ecosystem, "pip");
        assert_eq!(req.package, "httpie");
        assert_eq!(req.shell, Some("bash".to_string()));
        assert!(req.is_shell_request());
    }

    #[test]
    fn test_package_with_version_and_shell() {
        // npm:codex@1.0::cmd - specific version with shell
        let req = PackageRequest::parse("npm:codex@1.0::cmd").unwrap();
        assert_eq!(req.package, "codex");
        assert_eq!(req.version, Some("1.0".to_string()));
        assert_eq!(req.shell, Some("cmd".to_string()));
        assert!(req.is_shell_request());
    }

    #[test]
    fn test_package_with_runtime_version_and_shell() {
        // npm@20:codex::cmd - node 20 with codex, launch cmd
        let req = PackageRequest::parse("npm@20:codex::cmd").unwrap();
        assert_eq!(req.package, "codex");
        assert_eq!(req.shell, Some("cmd".to_string()));
        let spec = req.runtime_spec.unwrap();
        assert_eq!(spec.runtime, "node");
        assert_eq!(spec.version, "20");
    }

    #[test]
    fn test_executable_vs_shell_distinction() {
        // tsc is NOT a known shell, so it should be treated as executable
        let req = PackageRequest::parse("npm:typescript::tsc").unwrap();
        assert_eq!(req.executable, Some("tsc".to_string()));
        assert_eq!(req.shell, None);
        assert!(!req.is_shell_request());

        // cmd IS a known shell
        let req = PackageRequest::parse("npm:typescript::cmd").unwrap();
        assert_eq!(req.executable, None);
        assert_eq!(req.shell, Some("cmd".to_string()));
        assert!(req.is_shell_request());
    }

    #[test]
    fn test_all_known_shells() {
        for shell in KNOWN_SHELLS {
            let req = PackageRequest::parse(&format!("npm:test::{}", shell)).unwrap();
            assert_eq!(req.shell, Some(shell.to_string()));
            assert!(req.is_shell_request());
        }
    }
}
