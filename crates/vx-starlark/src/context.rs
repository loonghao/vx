//! ProviderContext - The API bridge between Starlark scripts and vx
//!
//! This module provides the `ProviderContext` struct that is injected into
//! Starlark scripts, giving them access to vx capabilities through a
//! sandboxed API.

use crate::error::{Error, Result};
use crate::sandbox::SandboxConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Platform information exposed to Starlark scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system: "windows", "macos", "linux"
    pub os: String,
    /// Architecture: "x64", "arm64", "x86"
    pub arch: String,
    /// Full target triple: "x86_64-pc-windows-msvc"
    pub target: String,
}

impl PlatformInfo {
    /// Create platform info from the current system
    pub fn current() -> Self {
        Self {
            os: Self::detect_os(),
            arch: Self::detect_arch(),
            target: Self::detect_target(),
        }
    }

    fn detect_os() -> String {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn detect_arch() -> String {
        if cfg!(target_arch = "x86_64") {
            "x64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "arm64".to_string()
        } else if cfg!(target_arch = "x86") {
            "x86".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn detect_target() -> String {
        // Get target at runtime
        std::env::var("TARGET").unwrap_or_else(|_| {
            // Fallback to compile-time target
            cfg!(target_arch)
                .then_some("x86_64")
                .unwrap_or("unknown")
                .to_string()
        })
    }
}

/// Version information returned by fetch_versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version string (e.g., "20.0.0")
    pub version: String,
    /// Whether this is an LTS version
    pub lts: bool,
    /// Whether this is a stable version
    pub stable: bool,
    /// Release date (optional)
    pub date: Option<String>,
}

impl VersionInfo {
    /// Create a new version info
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            lts: false,
            stable: true,
            date: None,
        }
    }

    /// Mark as LTS version
    pub fn with_lts(mut self, lts: bool) -> Self {
        self.lts = lts;
        self
    }

    /// Mark as stable version
    pub fn with_stable(mut self, stable: bool) -> Self {
        self.stable = stable;
        self
    }

    /// Set release date
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }
}

/// Installation result returned by install function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    /// Whether installation was successful
    pub success: bool,
    /// Path to the installed runtime
    pub install_path: PathBuf,
    /// Path to the executable
    pub executable_path: Option<PathBuf>,
    /// Optional message
    pub message: Option<String>,
}

impl InstallResult {
    /// Create a successful install result
    pub fn success(install_path: PathBuf) -> Self {
        Self {
            success: true,
            install_path,
            executable_path: None,
            message: None,
        }
    }

    /// Create a failed install result
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            install_path: PathBuf::new(),
            executable_path: None,
            message: Some(message.into()),
        }
    }

    /// Set executable path
    pub fn with_executable(mut self, path: PathBuf) -> Self {
        self.executable_path = Some(path);
        self
    }

    /// Set message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

/// Execution preparation result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionPrep {
    /// Override the executable path
    pub executable_override: Option<PathBuf>,
    /// Whether the runtime is ready for execution
    pub ready: bool,
    /// Optional message
    pub message: Option<String>,
    /// Environment variables to set
    pub env: HashMap<String, String>,
    /// Paths to prepend to PATH
    pub path_prepend: Vec<PathBuf>,
}

impl ExecutionPrep {
    /// Create a ready execution prep
    pub fn ready() -> Self {
        Self {
            ready: true,
            ..Default::default()
        }
    }

    /// Create a not-ready execution prep
    pub fn not_ready(message: impl Into<String>) -> Self {
        Self {
            ready: false,
            message: Some(message.into()),
            ..Default::default()
        }
    }
}

/// Path management for provider context
#[derive(Debug, Clone)]
pub struct PathManager {
    /// Provider name
    pub provider_name: String,
    /// Version being processed
    pub version: Option<String>,
    /// VX home directory
    pub vx_home: PathBuf,
    /// Store directory
    pub store_dir: PathBuf,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Temp directory
    pub temp_dir: PathBuf,
}

impl PathManager {
    /// Create a new path manager
    pub fn new(provider_name: &str, vx_home: PathBuf) -> Self {
        let store_dir = vx_home.join("store");
        let cache_dir = vx_home.join("cache");
        let temp_dir = vx_home.join("tmp");

        Self {
            provider_name: provider_name.to_string(),
            version: None,
            vx_home,
            store_dir,
            cache_dir,
            temp_dir,
        }
    }

    /// Set the current version
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    /// Get the install directory for a specific version
    pub fn install_dir(&self, version: &str) -> PathBuf {
        self.store_dir.join(&self.provider_name).join(version)
    }

    /// Get the current install directory
    pub fn current_install_dir(&self) -> Option<PathBuf> {
        self.version.as_ref().map(|v| self.install_dir(v))
    }

    /// Get the cache directory for downloads
    pub fn download_cache(&self) -> PathBuf {
        self.cache_dir.join("downloads")
    }

    /// Get a temp directory for this provider
    pub fn provider_temp(&self) -> PathBuf {
        self.temp_dir.join(&self.provider_name)
    }
}

/// The main context object exposed to Starlark scripts
///
/// This provides the `ctx` object that scripts use to interact with vx.
/// All methods are sandboxed according to the configured `SandboxConfig`.
#[derive(Debug)]
pub struct ProviderContext {
    /// Platform information
    pub platform: PlatformInfo,

    /// Path management
    pub paths: PathManager,

    /// Sandbox configuration
    pub sandbox: SandboxConfig,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Whether this is a dry run (no actual changes)
    pub dry_run: bool,

    /// Whether verbose logging is enabled
    pub verbose: bool,
}

impl ProviderContext {
    /// Create a new provider context
    pub fn new(provider_name: &str, vx_home: PathBuf) -> Self {
        Self {
            platform: PlatformInfo::current(),
            paths: PathManager::new(provider_name, vx_home),
            sandbox: SandboxConfig::default(),
            env: HashMap::new(),
            dry_run: false,
            verbose: false,
        }
    }

    /// Set sandbox configuration
    pub fn with_sandbox(mut self, sandbox: SandboxConfig) -> Self {
        self.sandbox = sandbox;
        self
    }

    /// Set version
    pub fn with_version(mut self, version: &str) -> Self {
        self.paths = self.paths.with_version(version);
        self
    }

    /// Set environment variables
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Set dry run mode
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Set verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    // === File System API ===

    /// Check if a path exists
    pub fn file_exists(&self, path: &PathBuf) -> Result<bool> {
        if !self.sandbox.is_path_allowed(path) {
            return Err(Error::FsAccessDenied { path: path.clone() });
        }
        Ok(path.exists())
    }

    /// Check if a directory exists
    pub fn dir_exists(&self, path: &PathBuf) -> Result<bool> {
        if !self.sandbox.is_path_allowed(path) {
            return Err(Error::FsAccessDenied { path: path.clone() });
        }
        Ok(path.exists() && path.is_dir())
    }

    /// Create a directory
    pub fn create_dir(&self, path: &PathBuf) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        if !self.sandbox.is_path_allowed(path) {
            return Err(Error::FsAccessDenied { path: path.clone() });
        }
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    /// Read a file
    pub fn read_file(&self, path: &PathBuf) -> Result<String> {
        if !self.sandbox.is_path_allowed(path) {
            return Err(Error::FsAccessDenied { path: path.clone() });
        }
        Ok(std::fs::read_to_string(path)?)
    }

    /// Write a file
    pub fn write_file(&self, path: &PathBuf, content: &str) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        if !self.sandbox.is_path_allowed(path) {
            return Err(Error::FsAccessDenied { path: path.clone() });
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// List directory contents
    pub fn list_dir(&self, path: &PathBuf) -> Result<Vec<PathBuf>> {
        if !self.sandbox.is_path_allowed(path) {
            return Err(Error::FsAccessDenied { path: path.clone() });
        }
        let entries = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        Ok(entries)
    }

    /// Remove a file or directory
    pub fn remove(&self, path: &PathBuf) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        if !self.sandbox.is_path_allowed(path) {
            return Err(Error::FsAccessDenied { path: path.clone() });
        }
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    // === Path Utilities ===

    /// Join path components
    pub fn path_join(&self, base: &PathBuf, name: &str) -> PathBuf {
        base.join(name)
    }

    /// Get the parent directory
    pub fn path_parent(&self, path: &PathBuf) -> Option<PathBuf> {
        path.parent().map(|p| p.to_path_buf())
    }

    /// Get the file name
    pub fn path_filename(&self, path: &PathBuf) -> Option<String> {
        path.file_name()
            .and_then(|n| n.to_str().map(|s| s.to_string()))
    }

    /// Get the file extension
    pub fn path_extension(&self, path: &PathBuf) -> Option<String> {
        path.extension()
            .and_then(|e| e.to_str().map(|s| s.to_string()))
    }

    // === String Utilities ===

    /// Join strings with a separator
    pub fn join_strings(&self, strings: &[String], separator: &str) -> String {
        strings.join(separator)
    }

    /// Split a string by separator
    pub fn split_string(&self, s: &str, separator: &str) -> Vec<String> {
        s.split(separator).map(|s| s.to_string()).collect()
    }

    /// Check if string matches a pattern
    pub fn matches(&self, s: &str, pattern: &str) -> bool {
        // Simple glob-like matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                s.starts_with(prefix) && s.ends_with(suffix)
            } else {
                s.contains(&pattern.replace('*', ""))
            }
        } else {
            s == pattern
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_info() {
        let platform = PlatformInfo::current();
        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());
    }

    #[test]
    fn test_version_info() {
        let v = VersionInfo::new("20.0.0").with_lts(true);
        assert_eq!(v.version, "20.0.0");
        assert!(v.lts);
    }

    #[test]
    fn test_path_manager() {
        let pm = PathManager::new("node", PathBuf::from("/tmp/vx"));
        assert_eq!(
            pm.install_dir("20.0.0"),
            PathBuf::from("/tmp/vx/store/node/20.0.0")
        );
    }

    #[test]
    fn test_provider_context_sandbox() {
        let ctx = ProviderContext::new("test", PathBuf::from("/tmp/vx"))
            .with_sandbox(SandboxConfig::restrictive());

        let path = PathBuf::from("/etc/passwd");
        assert!(ctx.file_exists(&path).is_err());
    }
}
