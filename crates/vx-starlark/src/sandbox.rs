//! Sandbox security configuration for Starlark providers
//!
//! This module implements a three-layer security model:
//! 1. Starlark language built-in safety (no imports, no direct I/O)
//! 2. vx sandbox restrictions (file system whitelist, HTTP whitelist)
//! 3. API permission control (context methods check permissions)
//!
//! # Declarative Permissions (Buck2 / Deno inspired)
//!
//! Provider scripts declare their required permissions at the top of the file:
//!
//! ```python
//! permissions = {
//!     "fs": ["~/.vx/store", "C:\\Program Files\\Microsoft Visual Studio"],
//!     "http": ["api.github.com", "aka.ms"],
//!     "exec": ["where", "powershell"],
//! }
//! ```
//!
//! The Rust side reads this `permissions` variable and builds a `SandboxConfig`
//! via `SandboxConfig::from_permissions()`.

use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;

/// Declarative permissions declared in provider.star
///
/// Inspired by Buck2's explicit dependency declarations and Deno's permission model.
/// Provider scripts declare what they need at the top of the file, and the Rust
/// side enforces these constraints.
///
/// # Example (in provider.star)
/// ```python
/// permissions = {
///     "fs": ["~/.vx/store", "C:\\Program Files\\Microsoft Visual Studio"],
///     "http": ["api.github.com", "aka.ms"],
///     "exec": ["where", "powershell"],
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct PermissionsDecl {
    /// File system paths the provider needs to access
    pub fs: Vec<String>,
    /// HTTP hosts the provider needs to contact
    pub http: Vec<String>,
    /// Commands the provider needs to execute
    pub exec: Vec<String>,
}

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
    /// Build a SandboxConfig from a declarative PermissionsDecl
    ///
    /// This is the Buck2/Deno-inspired approach: provider scripts declare
    /// what they need, and the Rust side enforces those constraints.
    ///
    /// Default HTTP hosts (common package registries) are always included.
    pub fn from_permissions(permissions: &PermissionsDecl) -> Result<Self> {
        // Start with a base config that has fs/http enabled but no whitelist
        let mut config = Self {
            fs_allowed_paths: vec![],
            http_allowed_hosts: Self::default_http_hosts(),
            allowed_commands: vec![],
            execution_timeout: Duration::from_secs(60),
            memory_limit: 64 * 1024 * 1024, // 64 MB
            enable_fs: !permissions.fs.is_empty(),
            enable_http: true,
            enable_execute: !permissions.exec.is_empty(),
            enable_network: true,
        };

        // Parse file system permissions (expand ~ to home dir)
        for path_str in &permissions.fs {
            let expanded = expand_home_dir(path_str);
            config.fs_allowed_paths.push(expanded);
        }

        // Add extra HTTP hosts from permissions
        for host in &permissions.http {
            if !config.http_allowed_hosts.contains(host) {
                config.http_allowed_hosts.push(host.clone());
            }
        }

        // Add allowed commands
        config.allowed_commands.extend(permissions.exec.clone());

        Ok(config)
    }

    /// Default HTTP hosts that are always allowed (common package registries)
    fn default_http_hosts() -> Vec<String> {
        vec![
            "api.github.com".to_string(),
            "github.com".to_string(),
            "nodejs.org".to_string(),
            "go.dev".to_string(),
            "pypi.org".to_string(),
            "static.rust-lang.org".to_string(),
            "registry.npmjs.org".to_string(),
        ]
    }

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
            // Support wildcard patterns like "*.nodejs.org"
            // Matches: "nodejs.org" (root domain) and "dist.nodejs.org" (subdomains)
            // Does NOT match: "evil-nodejs.org"
            if allowed.starts_with("*.") {
                let base = &allowed[2..]; // Remove "*.": "*.nodejs.org" → "nodejs.org"
                host == base || host.ends_with(&format!(".{}", base))
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

/// Expand `~` to the user's home directory
fn expand_home_dir(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            if path == "~" {
                return home;
            }
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

