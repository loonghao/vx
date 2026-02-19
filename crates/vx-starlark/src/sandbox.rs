//! Sandbox security configuration for Starlark providers
//!
//! This module implements a three-layer security model:
//! 1. Starlark language built-in safety (no imports, no direct I/O)
//! 2. vx sandbox restrictions (file system whitelist, HTTP whitelist)
//! 3. API permission control (context methods check permissions)

use std::path::PathBuf;
use std::time::Duration;

/// Sandbox configuration for Starlark script execution
#[derive(Clone, Debug)]
pub struct SandboxConfig {
    /// Allowed file system paths (whitelist)
    pub fs_allowed_paths: Vec<PathBuf>,

    /// Allowed HTTP hosts (whitelist)
    pub http_allowed_hosts: Vec<String>,

    /// Allowed commands for execution (whitelist, empty = disabled)
    pub allowed_commands: Vec<String>,

    /// Maximum execution time
    pub execution_timeout: Duration,

    /// Maximum memory usage (0 = unlimited)
    pub memory_limit: usize,

    /// Enable file system access
    pub enable_fs: bool,

    /// Enable HTTP requests
    pub enable_http: bool,

    /// Enable command execution
    pub enable_execute: bool,

    /// Enable network access (resolves to HTTP + DNS)
    pub enable_network: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            fs_allowed_paths: vec![],
            http_allowed_hosts: vec![],
            allowed_commands: vec![],
            execution_timeout: Duration::from_secs(30),
            memory_limit: 64 * 1024 * 1024, // 64 MB
            enable_fs: true,
            enable_http: true,
            enable_execute: false, // Disabled by default for security
            enable_network: true,
        }
    }
}

impl SandboxConfig {
    /// Create a new sandbox config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a restrictive sandbox (no I/O, no network)
    pub fn restrictive() -> Self {
        Self {
            fs_allowed_paths: vec![],
            http_allowed_hosts: vec![],
            allowed_commands: vec![],
            execution_timeout: Duration::from_secs(10),
            memory_limit: 16 * 1024 * 1024, // 16 MB
            enable_fs: false,
            enable_http: false,
            enable_execute: false,
            enable_network: false,
        }
    }

    /// Create a permissive sandbox for trusted providers
    pub fn permissive() -> Self {
        Self {
            fs_allowed_paths: vec![],
            http_allowed_hosts: vec![], // Empty = all allowed
            allowed_commands: vec![],
            execution_timeout: Duration::from_secs(300), // 5 minutes
            memory_limit: 256 * 1024 * 1024,             // 256 MB
            enable_fs: true,
            enable_http: true,
            enable_execute: true,
            enable_network: true,
        }
    }

    /// Add an allowed file system path
    pub fn allow_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.fs_allowed_paths.push(path.into());
        self
    }

    /// Add multiple allowed file system paths
    pub fn allow_paths(mut self, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        self.fs_allowed_paths.extend(paths);
        self
    }

    /// Add an allowed HTTP host
    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.http_allowed_hosts.push(host.into());
        self
    }

    /// Add multiple allowed HTTP hosts
    pub fn allow_hosts(mut self, hosts: impl IntoIterator<Item = String>) -> Self {
        self.http_allowed_hosts.extend(hosts);
        self
    }

    /// Add an allowed command
    pub fn allow_command(mut self, command: impl Into<String>) -> Self {
        self.allowed_commands.push(command.into());
        self
    }

    /// Set execution timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.execution_timeout = timeout;
        self
    }

    /// Set memory limit
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    /// Enable/disable file system access
    pub fn with_fs(mut self, enabled: bool) -> Self {
        self.enable_fs = enabled;
        self
    }

    /// Enable/disable HTTP requests
    pub fn with_http(mut self, enabled: bool) -> Self {
        self.enable_http = enabled;
        self
    }

    /// Enable/disable command execution
    pub fn with_execute(mut self, enabled: bool) -> Self {
        self.enable_execute = enabled;
        self
    }

    /// Check if a path is allowed
    pub fn is_path_allowed(&self, path: &PathBuf) -> bool {
        if !self.enable_fs {
            return false;
        }

        // If no whitelist, all paths are allowed (when fs is enabled)
        if self.fs_allowed_paths.is_empty() {
            return true;
        }

        // Check if path is within any allowed path
        self.fs_allowed_paths
            .iter()
            .any(|allowed| path.starts_with(allowed) || path == allowed)
    }

    /// Check if a host is allowed
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if !self.enable_http {
            return false;
        }

        // If no whitelist, all hosts are allowed (when http is enabled)
        if self.http_allowed_hosts.is_empty() {
            return true;
        }

        // Check if host matches any allowed pattern
        self.http_allowed_hosts.iter().any(|allowed| {
            // Support wildcard patterns like "*.github.com"
            if allowed.starts_with("*.") {
                let suffix = &allowed[1..]; // Remove first "*"
                host.ends_with(suffix)
            } else {
                host == allowed
            }
        })
    }

    /// Check if a command is allowed
    pub fn is_command_allowed(&self, command: &str) -> bool {
        if !self.enable_execute {
            return false;
        }

        // If no whitelist, no commands are allowed
        if self.allowed_commands.is_empty() {
            return false;
        }

        // Check if command is in whitelist
        self.allowed_commands
            .iter()
            .any(|allowed| command == allowed || command.starts_with(&format!("{} ", allowed)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SandboxConfig::default();
        assert!(config.enable_fs);
        assert!(config.enable_http);
        assert!(!config.enable_execute);
    }

    #[test]
    fn test_restrictive_config() {
        let config = SandboxConfig::restrictive();
        assert!(!config.enable_fs);
        assert!(!config.enable_http);
        assert!(!config.enable_execute);
    }

    #[test]
    fn test_permissive_config() {
        let config = SandboxConfig::permissive();
        assert!(config.enable_fs);
        assert!(config.enable_http);
        assert!(config.enable_execute);
    }

    #[test]
    fn test_path_whitelist() {
        let config = SandboxConfig::new()
            .allow_path("/tmp")
            .allow_path("/home/user/.vx");

        assert!(config.is_path_allowed(&PathBuf::from("/tmp")));
        assert!(config.is_path_allowed(&PathBuf::from("/tmp/subdir")));
        assert!(!config.is_path_allowed(&PathBuf::from("/etc")));
    }

    #[test]
    fn test_host_whitelist() {
        let config = SandboxConfig::new()
            .allow_host("github.com")
            .allow_host("*.nodejs.org");

        assert!(config.is_host_allowed("github.com"));
        assert!(config.is_host_allowed("nodejs.org"));
        assert!(config.is_host_allowed("dist.nodejs.org"));
        assert!(!config.is_host_allowed("example.com"));
    }

    #[test]
    fn test_command_whitelist() {
        let config = SandboxConfig::new()
            .with_execute(true)
            .allow_command("git")
            .allow_command("npm");

        assert!(config.is_command_allowed("git"));
        assert!(config.is_command_allowed("git clone"));
        assert!(config.is_command_allowed("npm install"));
        assert!(!config.is_command_allowed("rm"));
    }
}
