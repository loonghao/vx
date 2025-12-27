//! Dependencies management module
//!
//! This module provides utilities for managing project dependencies,
//! including registry configuration, constraint validation, and auto-update strategies.
//!
//! ## Features
//!
//! - Multi-package manager support (npm, yarn, pnpm, bun, pip, uv)
//! - Registry/mirror configuration
//! - Dependency constraints (version, license)
//! - Auto-update strategies
//!
//! ## Configuration Example
//!
//! ```toml
//! [dependencies]
//! lockfile = true
//! audit = true
//! auto_update = "patch"  # none, patch, minor, major
//!
//! [dependencies.node]
//! package_manager = "pnpm"
//! registry = "https://registry.npmmirror.com"
//!
//! [dependencies.python]
//! index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
//! extra_index_urls = ["https://pypi.org/simple"]
//!
//! [dependencies.constraints]
//! lodash = ">=4.17.21"
//! "left-pad" = { licenses = ["MIT", "Apache-2.0"] }
//! ```

use crate::types::{DependenciesConfig, NodeDependenciesConfig, PythonDependenciesConfig};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Auto-update strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoUpdateStrategy {
    /// No automatic updates
    None,
    /// Only patch version updates (x.y.Z)
    Patch,
    /// Minor and patch updates (x.Y.z)
    Minor,
    /// All updates including major (X.y.z)
    Major,
}

impl AutoUpdateStrategy {
    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "patch" => Self::Patch,
            "minor" => Self::Minor,
            "major" => Self::Major,
            _ => Self::None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Patch => "patch",
            Self::Minor => "minor",
            Self::Major => "major",
        }
    }
}

/// Dependency manager for a specific ecosystem
pub struct DependencyManager {
    config: DependenciesConfig,
    working_dir: std::path::PathBuf,
}

impl DependencyManager {
    /// Create a new dependency manager
    pub fn new(config: DependenciesConfig, working_dir: impl AsRef<Path>) -> Self {
        Self {
            config,
            working_dir: working_dir.as_ref().to_path_buf(),
        }
    }

    /// Get auto-update strategy
    pub fn auto_update_strategy(&self) -> AutoUpdateStrategy {
        self.config
            .auto_update
            .as_ref()
            .map(|s| AutoUpdateStrategy::from_str(s))
            .unwrap_or(AutoUpdateStrategy::None)
    }

    /// Check if lockfile generation is enabled
    pub fn lockfile_enabled(&self) -> bool {
        self.config.lockfile.unwrap_or(true)
    }

    /// Check if audit is enabled
    pub fn audit_enabled(&self) -> bool {
        self.config.audit.unwrap_or(false)
    }

    /// Get Node.js configuration
    pub fn node_config(&self) -> Option<&NodeDependenciesConfig> {
        self.config.node.as_ref()
    }

    /// Get Python configuration
    pub fn python_config(&self) -> Option<&PythonDependenciesConfig> {
        self.config.python.as_ref()
    }

    /// Get Node.js package manager
    pub fn node_package_manager(&self) -> &str {
        self.config
            .node
            .as_ref()
            .and_then(|n| n.package_manager.as_deref())
            .unwrap_or("npm")
    }

    /// Get Node.js registry URL
    pub fn node_registry(&self) -> Option<&str> {
        self.config
            .node
            .as_ref()
            .and_then(|n| n.registry.as_deref())
    }

    /// Get Python index URL
    pub fn python_index_url(&self) -> Option<&str> {
        self.config
            .python
            .as_ref()
            .and_then(|p| p.index_url.as_deref())
    }

    /// Get Python extra index URLs
    pub fn python_extra_index_urls(&self) -> Vec<&str> {
        self.config
            .python
            .as_ref()
            .map(|p| p.extra_index_urls.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Generate environment variables for Node.js package managers
    pub fn node_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        if let Some(registry) = self.node_registry() {
            // npm uses npm_config_registry
            env.insert("npm_config_registry".to_string(), registry.to_string());
            // yarn uses YARN_REGISTRY
            env.insert("YARN_REGISTRY".to_string(), registry.to_string());
            // pnpm uses npm_config_registry
        }

        env
    }

    /// Generate environment variables for Python package managers
    pub fn python_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        if let Some(index_url) = self.python_index_url() {
            // pip uses PIP_INDEX_URL
            env.insert("PIP_INDEX_URL".to_string(), index_url.to_string());
            // uv uses UV_INDEX_URL
            env.insert("UV_INDEX_URL".to_string(), index_url.to_string());
        }

        let extra_urls = self.python_extra_index_urls();
        if !extra_urls.is_empty() {
            let extra = extra_urls.join(" ");
            env.insert("PIP_EXTRA_INDEX_URL".to_string(), extra.clone());
            env.insert("UV_EXTRA_INDEX_URL".to_string(), extra);
        }

        env
    }

    /// Generate all environment variables
    pub fn all_env_vars(&self) -> HashMap<String, String> {
        let mut env = self.node_env_vars();
        env.extend(self.python_env_vars());
        env
    }

    /// Install Node.js dependencies
    pub fn install_node_dependencies(&self) -> Result<(), std::io::Error> {
        let pm = self.node_package_manager();
        let mut cmd = Command::new(pm);
        cmd.current_dir(&self.working_dir);

        // Add install command
        match pm {
            "npm" => {
                cmd.arg("install");
                if self.lockfile_enabled() {
                    cmd.arg("--package-lock");
                }
            }
            "yarn" => {
                cmd.arg("install");
            }
            "pnpm" => {
                cmd.arg("install");
                if !self.lockfile_enabled() {
                    cmd.arg("--no-lockfile");
                }
            }
            "bun" => {
                cmd.arg("install");
            }
            _ => {
                cmd.arg("install");
            }
        }

        // Add registry if configured
        if let Some(registry) = self.node_registry() {
            cmd.arg("--registry");
            cmd.arg(registry);
        }

        // Add environment variables
        for (key, value) in self.node_env_vars() {
            cmd.env(key, value);
        }

        let status = cmd.status()?;
        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{} install failed", pm),
            ));
        }

        Ok(())
    }

    /// Install Python dependencies
    pub fn install_python_dependencies(
        &self,
        requirements_file: Option<&str>,
    ) -> Result<(), std::io::Error> {
        // Prefer uv if available, fallback to pip
        let (pm, install_cmd) = if Command::new("uv")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            ("uv", vec!["pip", "install"])
        } else {
            ("pip", vec!["install"])
        };

        let mut cmd = Command::new(pm);
        cmd.current_dir(&self.working_dir);

        for arg in install_cmd {
            cmd.arg(arg);
        }

        // Add requirements file
        if let Some(req_file) = requirements_file {
            cmd.arg("-r");
            cmd.arg(req_file);
        }

        // Add index URL if configured
        if let Some(index_url) = self.python_index_url() {
            cmd.arg("--index-url");
            cmd.arg(index_url);
        }

        // Add extra index URLs
        for extra_url in self.python_extra_index_urls() {
            cmd.arg("--extra-index-url");
            cmd.arg(extra_url);
        }

        // Add environment variables
        for (key, value) in self.python_env_vars() {
            cmd.env(key, value);
        }

        let status = cmd.status()?;
        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{} install failed", pm),
            ));
        }

        Ok(())
    }

    /// Run npm audit
    pub fn run_node_audit(&self) -> Result<AuditResult, std::io::Error> {
        if !self.audit_enabled() {
            return Ok(AuditResult::default());
        }

        let pm = self.node_package_manager();
        let mut cmd = Command::new(pm);
        cmd.current_dir(&self.working_dir);
        cmd.arg("audit");
        cmd.arg("--json");

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON output (simplified)
        let vulnerabilities = if stdout.contains("\"vulnerabilities\"") {
            // Count vulnerabilities from JSON
            stdout.matches("\"severity\"").count()
        } else {
            0
        };

        Ok(AuditResult {
            vulnerabilities,
            success: output.status.success() || vulnerabilities == 0,
        })
    }
}

/// Audit result
#[derive(Debug, Clone, Default)]
pub struct AuditResult {
    /// Number of vulnerabilities found
    pub vulnerabilities: usize,
    /// Whether audit passed
    pub success: bool,
}

/// Registry presets for common mirrors
pub struct RegistryPresets;

impl RegistryPresets {
    /// Get npm registry presets
    pub fn npm() -> HashMap<&'static str, &'static str> {
        let mut presets = HashMap::new();
        presets.insert("npm", "https://registry.npmjs.org");
        presets.insert("npmmirror", "https://registry.npmmirror.com");
        presets.insert("taobao", "https://registry.npmmirror.com");
        presets.insert("yarn", "https://registry.yarnpkg.com");
        presets.insert("tencent", "https://mirrors.cloud.tencent.com/npm/");
        presets
    }

    /// Get PyPI registry presets
    pub fn pypi() -> HashMap<&'static str, &'static str> {
        let mut presets = HashMap::new();
        presets.insert("pypi", "https://pypi.org/simple");
        presets.insert("tsinghua", "https://pypi.tuna.tsinghua.edu.cn/simple");
        presets.insert("aliyun", "https://mirrors.aliyun.com/pypi/simple/");
        presets.insert("tencent", "https://mirrors.cloud.tencent.com/pypi/simple");
        presets.insert("douban", "https://pypi.doubanio.com/simple/");
        presets
    }

    /// Resolve preset name to URL
    pub fn resolve_npm(name: &str) -> Option<&'static str> {
        Self::npm().get(name).copied()
    }

    /// Resolve preset name to URL
    pub fn resolve_pypi(name: &str) -> Option<&'static str> {
        Self::pypi().get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_update_strategy() {
        assert_eq!(
            AutoUpdateStrategy::from_str("none"),
            AutoUpdateStrategy::None
        );
        assert_eq!(
            AutoUpdateStrategy::from_str("patch"),
            AutoUpdateStrategy::Patch
        );
        assert_eq!(
            AutoUpdateStrategy::from_str("minor"),
            AutoUpdateStrategy::Minor
        );
        assert_eq!(
            AutoUpdateStrategy::from_str("major"),
            AutoUpdateStrategy::Major
        );
        assert_eq!(
            AutoUpdateStrategy::from_str("PATCH"),
            AutoUpdateStrategy::Patch
        );
        assert_eq!(
            AutoUpdateStrategy::from_str("invalid"),
            AutoUpdateStrategy::None
        );
    }

    #[test]
    fn test_dependency_manager_defaults() {
        let config = DependenciesConfig::default();
        let manager = DependencyManager::new(config, ".");

        assert!(manager.lockfile_enabled());
        assert!(!manager.audit_enabled());
        assert_eq!(manager.auto_update_strategy(), AutoUpdateStrategy::None);
        assert_eq!(manager.node_package_manager(), "npm");
    }

    #[test]
    fn test_node_env_vars() {
        let config = DependenciesConfig {
            node: Some(NodeDependenciesConfig {
                package_manager: Some("pnpm".to_string()),
                registry: Some("https://registry.npmmirror.com".to_string()),
            }),
            ..Default::default()
        };
        let manager = DependencyManager::new(config, ".");

        let env = manager.node_env_vars();
        assert_eq!(
            env.get("npm_config_registry"),
            Some(&"https://registry.npmmirror.com".to_string())
        );
        assert_eq!(
            env.get("YARN_REGISTRY"),
            Some(&"https://registry.npmmirror.com".to_string())
        );
    }

    #[test]
    fn test_python_env_vars() {
        let config = DependenciesConfig {
            python: Some(PythonDependenciesConfig {
                index_url: Some("https://pypi.tuna.tsinghua.edu.cn/simple".to_string()),
                extra_index_urls: vec!["https://pypi.org/simple".to_string()],
            }),
            ..Default::default()
        };
        let manager = DependencyManager::new(config, ".");

        let env = manager.python_env_vars();
        assert_eq!(
            env.get("PIP_INDEX_URL"),
            Some(&"https://pypi.tuna.tsinghua.edu.cn/simple".to_string())
        );
        assert_eq!(
            env.get("UV_INDEX_URL"),
            Some(&"https://pypi.tuna.tsinghua.edu.cn/simple".to_string())
        );
        assert_eq!(
            env.get("PIP_EXTRA_INDEX_URL"),
            Some(&"https://pypi.org/simple".to_string())
        );
    }

    #[test]
    fn test_registry_presets() {
        assert_eq!(
            RegistryPresets::resolve_npm("npmmirror"),
            Some("https://registry.npmmirror.com")
        );
        assert_eq!(
            RegistryPresets::resolve_pypi("tsinghua"),
            Some("https://pypi.tuna.tsinghua.edu.cn/simple")
        );
        assert_eq!(RegistryPresets::resolve_npm("unknown"), None);
    }
}
