//! Dependencies management module
//!
//! This module provides utilities for managing project dependencies,
//! including registry configuration, constraint validation, and auto-update strategies.
//!
//! ## Features
//!
//! - Multi-package manager support (npm, yarn, pnpm, bun, pip, uv, go, conan, vcpkg)
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
//! [dependencies.go]
//! proxy = "https://goproxy.cn,direct"
//! private = "github.com/mycompany/*"
//!
//! [dependencies.cpp]
//! package_manager = "vcpkg"
//! vcpkg_triplet = "x64-linux"
//! cmake_generator = "Ninja"
//!
//! [dependencies.constraints]
//! lodash = ">=4.17.21"
//! "left-pad" = { licenses = ["MIT", "Apache-2.0"] }
//! ```

use crate::types::{
    CppDependenciesConfig, DependenciesConfig, GoDependenciesConfig, NodeDependenciesConfig,
    PythonDependenciesConfig,
};
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
    pub fn parse(s: &str) -> Self {
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
            .map(|s| AutoUpdateStrategy::parse(s))
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

    /// Get Go configuration
    pub fn go_config(&self) -> Option<&GoDependenciesConfig> {
        self.config.go.as_ref()
    }

    /// Get C++ configuration
    pub fn cpp_config(&self) -> Option<&CppDependenciesConfig> {
        self.config.cpp.as_ref()
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

    /// Get Go proxy URL
    pub fn go_proxy(&self) -> Option<&str> {
        self.config.go.as_ref().and_then(|g| g.proxy.as_deref())
    }

    /// Get Go private modules pattern
    pub fn go_private(&self) -> Option<&str> {
        self.config.go.as_ref().and_then(|g| g.private.as_deref())
    }

    /// Get C++ package manager
    pub fn cpp_package_manager(&self) -> &str {
        self.config
            .cpp
            .as_ref()
            .and_then(|c| c.package_manager.as_deref())
            .unwrap_or("cmake")
    }

    /// Get vcpkg triplet
    pub fn vcpkg_triplet(&self) -> Option<&str> {
        self.config
            .cpp
            .as_ref()
            .and_then(|c| c.vcpkg_triplet.as_deref())
    }

    /// Get CMake generator
    pub fn cmake_generator(&self) -> Option<&str> {
        self.config
            .cpp
            .as_ref()
            .and_then(|c| c.cmake_generator.as_deref())
    }

    /// Get CMake build type
    pub fn cmake_build_type(&self) -> &str {
        self.config
            .cpp
            .as_ref()
            .and_then(|c| c.cmake_build_type.as_deref())
            .unwrap_or("Release")
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

    /// Generate environment variables for Go
    pub fn go_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        if let Some(proxy) = self.go_proxy() {
            env.insert("GOPROXY".to_string(), proxy.to_string());
        }

        if let Some(private) = self.go_private() {
            env.insert("GOPRIVATE".to_string(), private.to_string());
        }

        if let Some(go_config) = self.go_config() {
            if let Some(sumdb) = &go_config.sumdb {
                env.insert("GOSUMDB".to_string(), sumdb.clone());
            }
            if let Some(nosumdb) = &go_config.nosumdb {
                env.insert("GONOSUMDB".to_string(), nosumdb.clone());
            }
            if go_config.vendor.unwrap_or(false) {
                env.insert("GOFLAGS".to_string(), "-mod=vendor".to_string());
            } else if let Some(mod_mode) = &go_config.mod_mode {
                env.insert("GOFLAGS".to_string(), format!("-mod={}", mod_mode));
            }
        }

        env
    }

    /// Generate environment variables for C++
    pub fn cpp_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        if let Some(cpp_config) = self.cpp_config() {
            if let Some(vcpkg_root) = &cpp_config.vcpkg_root {
                env.insert("VCPKG_ROOT".to_string(), vcpkg_root.clone());
            }
            if let Some(triplet) = &cpp_config.vcpkg_triplet {
                env.insert("VCPKG_DEFAULT_TRIPLET".to_string(), triplet.clone());
            }
            if let Some(generator) = &cpp_config.cmake_generator {
                env.insert("CMAKE_GENERATOR".to_string(), generator.clone());
            }
            if let Some(build_type) = &cpp_config.cmake_build_type {
                env.insert("CMAKE_BUILD_TYPE".to_string(), build_type.clone());
            }
            if let Some(std) = &cpp_config.std {
                env.insert("CMAKE_CXX_STANDARD".to_string(), std.clone());
            }
        }

        env
    }

    /// Generate all environment variables
    pub fn all_env_vars(&self) -> HashMap<String, String> {
        let mut env = self.node_env_vars();
        env.extend(self.python_env_vars());
        env.extend(self.go_env_vars());
        env.extend(self.cpp_env_vars());
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
            return Err(std::io::Error::other(format!("{} install failed", pm)));
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
            return Err(std::io::Error::other(format!("{} install failed", pm)));
        }

        Ok(())
    }

    /// Install Go dependencies
    pub fn install_go_dependencies(&self) -> Result<(), std::io::Error> {
        let mut cmd = Command::new("go");
        cmd.current_dir(&self.working_dir);
        cmd.args(["mod", "download"]);

        // Add environment variables
        for (key, value) in self.go_env_vars() {
            cmd.env(key, value);
        }

        let status = cmd.status()?;
        if !status.success() {
            return Err(std::io::Error::other("go mod download failed"));
        }

        // Optionally run go mod tidy
        let mut tidy_cmd = Command::new("go");
        tidy_cmd.current_dir(&self.working_dir);
        tidy_cmd.args(["mod", "tidy"]);

        for (key, value) in self.go_env_vars() {
            tidy_cmd.env(key, value);
        }

        let _ = tidy_cmd.status(); // Ignore tidy errors

        Ok(())
    }

    /// Install C++ dependencies using configured package manager
    pub fn install_cpp_dependencies(&self) -> Result<(), std::io::Error> {
        let pm = self.cpp_package_manager();

        match pm {
            "vcpkg" => self.install_vcpkg_dependencies(),
            "conan" => self.install_conan_dependencies(),
            _ => self.configure_cmake(),
        }
    }

    /// Install dependencies using vcpkg
    fn install_vcpkg_dependencies(&self) -> Result<(), std::io::Error> {
        let mut cmd = Command::new("vcpkg");
        cmd.current_dir(&self.working_dir);
        cmd.arg("install");

        // Add triplet if configured
        if let Some(triplet) = self.vcpkg_triplet() {
            cmd.args(["--triplet", triplet]);
        }

        // Add environment variables
        for (key, value) in self.cpp_env_vars() {
            cmd.env(key, value);
        }

        let status = cmd.status()?;
        if !status.success() {
            return Err(std::io::Error::other("vcpkg install failed"));
        }

        Ok(())
    }

    /// Install dependencies using Conan
    fn install_conan_dependencies(&self) -> Result<(), std::io::Error> {
        let mut cmd = Command::new("conan");
        cmd.current_dir(&self.working_dir);
        cmd.args(["install", ".", "--build=missing"]);

        // Add remote if configured
        if let Some(cpp_config) = self.cpp_config() {
            if let Some(remote) = &cpp_config.conan_remote {
                // First add the remote
                let _ = Command::new("conan")
                    .args(["remote", "add", "custom", remote, "--force"])
                    .status();
            }
        }

        // Add environment variables
        for (key, value) in self.cpp_env_vars() {
            cmd.env(key, value);
        }

        let status = cmd.status()?;
        if !status.success() {
            return Err(std::io::Error::other("conan install failed"));
        }

        Ok(())
    }

    /// Configure CMake project
    fn configure_cmake(&self) -> Result<(), std::io::Error> {
        let mut cmd = Command::new("cmake");
        cmd.current_dir(&self.working_dir);
        cmd.args(["-B", "build"]);

        // Add generator if configured
        if let Some(generator) = self.cmake_generator() {
            cmd.args(["-G", generator]);
        }

        // Add build type
        cmd.arg(format!("-DCMAKE_BUILD_TYPE={}", self.cmake_build_type()));

        // Add C++ standard if configured
        if let Some(cpp_config) = self.cpp_config() {
            if let Some(std) = &cpp_config.std {
                cmd.arg(format!("-DCMAKE_CXX_STANDARD={}", std));
            }

            // Add custom CMake options
            for (key, value) in &cpp_config.cmake_options {
                cmd.arg(format!("-D{}={}", key, value));
            }
        }

        // Add environment variables
        for (key, value) in self.cpp_env_vars() {
            cmd.env(key, value);
        }

        let status = cmd.status()?;
        if !status.success() {
            return Err(std::io::Error::other("cmake configure failed"));
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

    /// Get Go proxy presets
    pub fn goproxy() -> HashMap<&'static str, &'static str> {
        let mut presets = HashMap::new();
        presets.insert("default", "https://proxy.golang.org,direct");
        presets.insert("goproxy.cn", "https://goproxy.cn,direct");
        presets.insert("goproxy.io", "https://goproxy.io,direct");
        presets.insert("aliyun", "https://mirrors.aliyun.com/goproxy/,direct");
        presets.insert("tencent", "https://mirrors.cloud.tencent.com/go/,direct");
        presets.insert("athens", "https://athens.azurefd.net");
        presets
    }

    /// Get Conan remote presets
    pub fn conan() -> HashMap<&'static str, &'static str> {
        let mut presets = HashMap::new();
        presets.insert("conancenter", "https://center.conan.io");
        presets.insert(
            "bincrafters",
            "https://bincrafters.jfrog.io/artifactory/api/conan/public-conan",
        );
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

    /// Resolve Go proxy preset name to URL
    pub fn resolve_goproxy(name: &str) -> Option<&'static str> {
        Self::goproxy().get(name).copied()
    }

    /// Resolve Conan remote preset name to URL
    pub fn resolve_conan(name: &str) -> Option<&'static str> {
        Self::conan().get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_update_strategy() {
        assert_eq!(AutoUpdateStrategy::parse("none"), AutoUpdateStrategy::None);
        assert_eq!(
            AutoUpdateStrategy::parse("patch"),
            AutoUpdateStrategy::Patch
        );
        assert_eq!(
            AutoUpdateStrategy::parse("minor"),
            AutoUpdateStrategy::Minor
        );
        assert_eq!(
            AutoUpdateStrategy::parse("major"),
            AutoUpdateStrategy::Major
        );
        assert_eq!(
            AutoUpdateStrategy::parse("PATCH"),
            AutoUpdateStrategy::Patch
        );
        assert_eq!(
            AutoUpdateStrategy::parse("invalid"),
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
        assert_eq!(manager.cpp_package_manager(), "cmake");
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
    fn test_go_env_vars() {
        let config = DependenciesConfig {
            go: Some(GoDependenciesConfig {
                proxy: Some("https://goproxy.cn,direct".to_string()),
                private: Some("github.com/mycompany/*".to_string()),
                sumdb: Some("sum.golang.org".to_string()),
                nosumdb: None,
                vendor: Some(false),
                mod_mode: Some("readonly".to_string()),
            }),
            ..Default::default()
        };
        let manager = DependencyManager::new(config, ".");

        let env = manager.go_env_vars();
        assert_eq!(
            env.get("GOPROXY"),
            Some(&"https://goproxy.cn,direct".to_string())
        );
        assert_eq!(
            env.get("GOPRIVATE"),
            Some(&"github.com/mycompany/*".to_string())
        );
        assert_eq!(env.get("GOSUMDB"), Some(&"sum.golang.org".to_string()));
        assert_eq!(env.get("GOFLAGS"), Some(&"-mod=readonly".to_string()));
    }

    #[test]
    fn test_go_vendor_mode() {
        let config = DependenciesConfig {
            go: Some(GoDependenciesConfig {
                proxy: None,
                private: None,
                sumdb: None,
                nosumdb: None,
                vendor: Some(true),
                mod_mode: None,
            }),
            ..Default::default()
        };
        let manager = DependencyManager::new(config, ".");

        let env = manager.go_env_vars();
        assert_eq!(env.get("GOFLAGS"), Some(&"-mod=vendor".to_string()));
    }

    #[test]
    fn test_cpp_env_vars() {
        let mut cmake_options = HashMap::new();
        cmake_options.insert("BUILD_TESTS".to_string(), "ON".to_string());

        let config = DependenciesConfig {
            cpp: Some(CppDependenciesConfig {
                package_manager: Some("vcpkg".to_string()),
                conan_remote: None,
                vcpkg_root: Some("/opt/vcpkg".to_string()),
                vcpkg_triplet: Some("x64-linux".to_string()),
                cmake_generator: Some("Ninja".to_string()),
                cmake_build_type: Some("Release".to_string()),
                cmake_options,
                std: Some("17".to_string()),
                compiler: Some("clang".to_string()),
            }),
            ..Default::default()
        };
        let manager = DependencyManager::new(config, ".");

        let env = manager.cpp_env_vars();
        assert_eq!(env.get("VCPKG_ROOT"), Some(&"/opt/vcpkg".to_string()));
        assert_eq!(
            env.get("VCPKG_DEFAULT_TRIPLET"),
            Some(&"x64-linux".to_string())
        );
        assert_eq!(env.get("CMAKE_GENERATOR"), Some(&"Ninja".to_string()));
        assert_eq!(env.get("CMAKE_BUILD_TYPE"), Some(&"Release".to_string()));
        assert_eq!(env.get("CMAKE_CXX_STANDARD"), Some(&"17".to_string()));
    }

    #[test]
    fn test_cpp_package_manager() {
        let config = DependenciesConfig {
            cpp: Some(CppDependenciesConfig {
                package_manager: Some("conan".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let manager = DependencyManager::new(config, ".");
        assert_eq!(manager.cpp_package_manager(), "conan");

        // Default to cmake
        let default_config = DependenciesConfig::default();
        let default_manager = DependencyManager::new(default_config, ".");
        assert_eq!(default_manager.cpp_package_manager(), "cmake");
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
        assert_eq!(
            RegistryPresets::resolve_goproxy("goproxy.cn"),
            Some("https://goproxy.cn,direct")
        );
        assert_eq!(
            RegistryPresets::resolve_conan("conancenter"),
            Some("https://center.conan.io")
        );
        assert_eq!(RegistryPresets::resolve_npm("unknown"), None);
        assert_eq!(RegistryPresets::resolve_goproxy("unknown"), None);
    }

    #[test]
    fn test_all_env_vars() {
        let config = DependenciesConfig {
            node: Some(NodeDependenciesConfig {
                registry: Some("https://registry.npmmirror.com".to_string()),
                ..Default::default()
            }),
            python: Some(PythonDependenciesConfig {
                index_url: Some("https://pypi.tuna.tsinghua.edu.cn/simple".to_string()),
                ..Default::default()
            }),
            go: Some(GoDependenciesConfig {
                proxy: Some("https://goproxy.cn,direct".to_string()),
                ..Default::default()
            }),
            cpp: Some(CppDependenciesConfig {
                cmake_generator: Some("Ninja".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let manager = DependencyManager::new(config, ".");

        let env = manager.all_env_vars();
        assert!(env.contains_key("npm_config_registry"));
        assert!(env.contains_key("PIP_INDEX_URL"));
        assert!(env.contains_key("GOPROXY"));
        assert!(env.contains_key("CMAKE_GENERATOR"));
    }
}
