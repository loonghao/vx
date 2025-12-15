//! Context types for dependency injection
//!
//! These contexts provide all external dependencies needed by runtimes,
//! allowing for easy testing through mock implementations.

use crate::traits::{CommandExecutor, FileSystem, HttpClient, Installer, PathProvider};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Configuration for runtime operations
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Whether to automatically install missing runtimes
    pub auto_install: bool,
    /// Whether to include prerelease versions
    pub include_prerelease: bool,
    /// Installation timeout
    pub install_timeout: Duration,
    /// Whether to verify checksums
    pub verify_checksum: bool,
    /// Whether to use verbose output
    pub verbose: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            include_prerelease: false,
            install_timeout: Duration::from_secs(300), // 5 minutes
            verify_checksum: true,
            verbose: false,
        }
    }
}

/// Context for runtime operations (install, fetch versions, etc.)
///
/// This context provides all dependencies needed for runtime operations,
/// allowing for easy mocking in tests.
pub struct RuntimeContext {
    /// Path provider for directory management
    pub paths: Arc<dyn PathProvider>,
    /// HTTP client for network requests
    pub http: Arc<dyn HttpClient>,
    /// File system operations
    pub fs: Arc<dyn FileSystem>,
    /// Archive installer
    pub installer: Arc<dyn Installer>,
    /// Configuration
    pub config: RuntimeConfig,
}

impl RuntimeContext {
    /// Create a new runtime context
    pub fn new(
        paths: Arc<dyn PathProvider>,
        http: Arc<dyn HttpClient>,
        fs: Arc<dyn FileSystem>,
        installer: Arc<dyn Installer>,
    ) -> Self {
        Self {
            paths,
            http,
            fs,
            installer,
            config: RuntimeConfig::default(),
        }
    }

    /// Create a new runtime context with custom config
    pub fn with_config(mut self, config: RuntimeConfig) -> Self {
        self.config = config;
        self
    }
}

/// Context for command execution
///
/// This context provides all dependencies needed for executing commands,
/// allowing for easy mocking in tests.
pub struct ExecutionContext {
    /// Working directory for the command
    pub working_dir: Option<PathBuf>,
    /// Environment variables to set
    pub env: HashMap<String, String>,
    /// Whether to capture stdout/stderr
    pub capture_output: bool,
    /// Command timeout
    pub timeout: Option<Duration>,
    /// Command executor
    pub executor: Arc<dyn CommandExecutor>,
}

impl ExecutionContext {
    /// Create a new execution context with an executor
    pub fn new(executor: Arc<dyn CommandExecutor>) -> Self {
        Self {
            working_dir: None,
            env: HashMap::new(),
            capture_output: false,
            timeout: None,
            executor,
        }
    }

    /// Set working directory
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set multiple environment variables
    pub fn with_envs(mut self, envs: HashMap<String, String>) -> Self {
        self.env.extend(envs);
        self
    }

    /// Enable output capture
    pub fn with_capture_output(mut self, capture: bool) -> Self {
        self.capture_output = capture;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}
