//! Testing utilities and mock implementations
//!
//! This module provides mock implementations of all traits for unit testing.
//!
//! # Example
//!
//! ```rust,no_run
//! use vx_runtime::testing::{mock_context, MockHttpClient};
//!
//! #[tokio::test]
//! async fn test_fetch_versions() {
//!     let ctx = mock_context();
//!
//!     // Set up mock HTTP response
//!     // ctx.http.mock_response("https://api.example.com", "...");
//!
//!     // Test your runtime
//!     // let runtime = MyRuntime::new();
//!     // let versions = runtime.fetch_versions(&ctx).await.unwrap();
//! }
//! ```

use crate::context::{ExecutionContext, RuntimeConfig, RuntimeContext};
use crate::traits::{CommandExecutor, FileSystem, HttpClient, Installer, PathProvider};
use crate::types::ExecutionResult;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

// ============================================================================
// Mock Path Provider
// ============================================================================

/// Mock path provider for testing
pub struct MockPathProvider {
    base_dir: PathBuf,
}

impl MockPathProvider {
    /// Create a new mock path provider with a base directory
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }
}

impl PathProvider for MockPathProvider {
    fn vx_home(&self) -> PathBuf {
        self.base_dir.clone()
    }

    fn store_dir(&self) -> PathBuf {
        self.base_dir.join("store")
    }

    fn envs_dir(&self) -> PathBuf {
        self.base_dir.join("envs")
    }

    fn bin_dir(&self) -> PathBuf {
        self.base_dir.join("bin")
    }

    fn cache_dir(&self) -> PathBuf {
        self.base_dir.join("cache")
    }

    fn config_dir(&self) -> PathBuf {
        self.base_dir.join("config")
    }

    fn runtime_store_dir(&self, name: &str) -> PathBuf {
        self.store_dir().join(name)
    }

    fn version_store_dir(&self, name: &str, version: &str) -> PathBuf {
        self.runtime_store_dir(name).join(version)
    }

    fn executable_path(&self, name: &str, version: &str) -> PathBuf {
        let exe_name = if cfg!(windows) {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };
        self.version_store_dir(name, version)
            .join("bin")
            .join(exe_name)
    }

    fn env_dir(&self, env_name: &str) -> PathBuf {
        self.envs_dir().join(env_name)
    }

    // ========== npm-tools paths ==========

    fn npm_tools_dir(&self) -> PathBuf {
        self.base_dir.join("npm-tools")
    }

    fn npm_tool_dir(&self, package_name: &str) -> PathBuf {
        self.npm_tools_dir().join(package_name)
    }

    fn npm_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.npm_tool_dir(package_name).join(version)
    }

    fn npm_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.npm_tool_version_dir(package_name, version).join("bin")
    }

    // ========== pip-tools paths ==========

    fn pip_tools_dir(&self) -> PathBuf {
        self.base_dir.join("pip-tools")
    }

    fn pip_tool_dir(&self, package_name: &str) -> PathBuf {
        self.pip_tools_dir().join(package_name)
    }

    fn pip_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.pip_tool_dir(package_name).join(version)
    }

    fn pip_tool_venv_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.pip_tool_version_dir(package_name, version)
            .join("venv")
    }

    fn pip_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        let venv_dir = self.pip_tool_venv_dir(package_name, version);
        if cfg!(windows) {
            venv_dir.join("Scripts")
        } else {
            venv_dir.join("bin")
        }
    }

    // ========== RFC 0025: Global Package Isolation ==========

    fn packages_dir(&self) -> PathBuf {
        self.base_dir.join("packages")
    }

    fn shims_dir(&self) -> PathBuf {
        self.base_dir.join("shims")
    }

    fn packages_registry_file(&self) -> PathBuf {
        self.config_dir().join("global-packages.json")
    }

    fn ecosystem_packages_dir(&self, ecosystem: &str) -> PathBuf {
        self.packages_dir().join(ecosystem)
    }

    fn global_package_dir(&self, ecosystem: &str, package: &str, version: &str) -> PathBuf {
        self.ecosystem_packages_dir(ecosystem)
            .join(vx_paths::normalize_package_name(package))
            .join(version)
    }

    fn global_package_bin_dir(&self, ecosystem: &str, package: &str, version: &str) -> PathBuf {
        self.global_package_dir(ecosystem, package, version)
            .join("bin")
    }
}

// ============================================================================
// Mock HTTP Client
// ============================================================================

/// Mock HTTP client for testing
pub struct MockHttpClient {
    responses: RwLock<HashMap<String, String>>,
    json_responses: RwLock<HashMap<String, serde_json::Value>>,
}

impl MockHttpClient {
    /// Create a new mock HTTP client
    pub fn new() -> Self {
        Self {
            responses: RwLock::new(HashMap::new()),
            json_responses: RwLock::new(HashMap::new()),
        }
    }

    /// Set a mock response for a URL
    pub fn mock_response(&self, url: &str, response: impl Into<String>) {
        self.responses
            .write()
            .unwrap()
            .insert(url.to_string(), response.into());
    }

    /// Set a mock JSON response for a URL
    pub fn mock_json(&self, url: &str, response: serde_json::Value) {
        self.json_responses
            .write()
            .unwrap()
            .insert(url.to_string(), response);
    }
}

impl Default for MockHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn get(&self, url: &str) -> Result<String> {
        self.responses
            .read()
            .unwrap()
            .get(url)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No mock response for URL: {}", url))
    }

    async fn get_json_value(&self, url: &str) -> Result<serde_json::Value> {
        self.json_responses
            .read()
            .unwrap()
            .get(url)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No mock JSON response for URL: {}", url))
    }

    async fn download(&self, _url: &str, _dest: &Path) -> Result<()> {
        // Mock download - just create an empty file
        Ok(())
    }

    async fn download_with_progress(
        &self,
        url: &str,
        dest: &Path,
        _on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    ) -> Result<()> {
        self.download(url, dest).await
    }
}

// ============================================================================
// Mock File System
// ============================================================================

/// Mock file system for testing
pub struct MockFileSystem {
    files: RwLock<HashMap<PathBuf, Vec<u8>>>,
    dirs: RwLock<HashSet<PathBuf>>,
}

impl MockFileSystem {
    /// Create a new mock file system
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
            dirs: RwLock::new(HashSet::new()),
        }
    }

    /// Add a file to the mock file system
    pub fn add_file(&self, path: impl Into<PathBuf>, content: impl Into<Vec<u8>>) {
        let path = path.into();
        if let Some(parent) = path.parent() {
            self.add_dir(parent);
        }
        self.files.write().unwrap().insert(path, content.into());
    }

    /// Add a directory to the mock file system
    pub fn add_dir(&self, path: impl Into<PathBuf>) {
        let path = path.into();
        let mut dirs = self.dirs.write().unwrap();

        // Add all parent directories
        let mut current = path.clone();
        while current.parent().is_some() {
            dirs.insert(current.clone());
            current = current.parent().unwrap().to_path_buf();
        }
        dirs.insert(path);
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for MockFileSystem {
    fn exists(&self, path: &Path) -> bool {
        self.files.read().unwrap().contains_key(path) || self.dirs.read().unwrap().contains(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.read().unwrap().contains(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.read().unwrap().contains_key(path)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.add_dir(path);
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        let mut dirs = self.dirs.write().unwrap();
        let mut files = self.files.write().unwrap();

        // Remove all files and dirs under this path
        dirs.retain(|p| !p.starts_with(path));
        files.retain(|p, _| !p.starts_with(path));

        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        self.files.write().unwrap().remove(path);
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let dirs = self.dirs.read().unwrap();
        let files = self.files.read().unwrap();

        let mut entries: Vec<PathBuf> = dirs
            .iter()
            .filter(|p| p.parent() == Some(path))
            .cloned()
            .collect();

        entries.extend(files.keys().filter(|p| p.parent() == Some(path)).cloned());

        Ok(entries)
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        let files = self.files.read().unwrap();
        let content = files
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("File not found: {:?}", path))?;
        String::from_utf8(content.clone()).map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.files
            .read()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("File not found: {:?}", path))
    }

    fn write(&self, path: &Path, content: &str) -> Result<()> {
        self.add_file(path, content.as_bytes());
        Ok(())
    }

    fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<()> {
        self.add_file(path, content);
        Ok(())
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<()> {
        let content = self.read(from)?;
        self.write_bytes(to, &content)
    }

    fn hard_link(&self, src: &Path, dst: &Path) -> Result<()> {
        // In mock, just copy the file reference
        let content = self.read(src)?;
        self.write_bytes(dst, &content)
    }

    fn symlink(&self, src: &Path, dst: &Path) -> Result<()> {
        // In mock, just copy the file reference
        let content = self.read(src)?;
        self.write_bytes(dst, &content)
    }

    #[cfg(unix)]
    fn set_permissions(&self, _path: &Path, _mode: u32) -> Result<()> {
        Ok(())
    }
}

// ============================================================================
// Mock Command Executor
// ============================================================================

/// Mock command executor for testing
pub struct MockCommandExecutor {
    results: RwLock<HashMap<String, ExecutionResult>>,
    programs: RwLock<HashSet<String>>,
}

impl MockCommandExecutor {
    /// Create a new mock command executor
    pub fn new() -> Self {
        Self {
            results: RwLock::new(HashMap::new()),
            programs: RwLock::new(HashSet::new()),
        }
    }

    /// Set the result for a command
    pub fn mock_result(&self, program: &str, result: ExecutionResult) {
        self.results
            .write()
            .unwrap()
            .insert(program.to_string(), result);
    }

    /// Add a program to the mock PATH
    pub fn add_program(&self, program: &str) {
        self.programs.write().unwrap().insert(program.to_string());
    }
}

impl Default for MockCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandExecutor for MockCommandExecutor {
    async fn execute(
        &self,
        program: &str,
        _args: &[String],
        _working_dir: Option<&Path>,
        _env: &HashMap<String, String>,
        _capture_output: bool,
    ) -> Result<ExecutionResult> {
        self.results
            .read()
            .unwrap()
            .get(program)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No mock result for program: {}", program))
    }

    fn which(&self, program: &str) -> Option<PathBuf> {
        if self.programs.read().unwrap().contains(program) {
            Some(PathBuf::from(format!("/usr/bin/{}", program)))
        } else {
            None
        }
    }
}

// ============================================================================
// Mock Installer
// ============================================================================

/// Mock installer for testing
pub struct MockInstaller {
    fs: Arc<MockFileSystem>,
}

impl MockInstaller {
    /// Create a new mock installer
    pub fn new() -> Self {
        Self {
            fs: Arc::new(MockFileSystem::new()),
        }
    }

    /// Create a mock installer with a shared file system
    pub fn with_fs(fs: Arc<MockFileSystem>) -> Self {
        Self { fs }
    }
}

impl Default for MockInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Installer for MockInstaller {
    async fn extract(&self, _archive: &Path, dest: &Path) -> Result<()> {
        // Mock extraction - just create the directory
        self.fs.create_dir_all(dest)?;
        Ok(())
    }

    async fn download_and_extract(&self, _url: &str, dest: &Path) -> Result<()> {
        // Mock download and extraction
        self.fs.create_dir_all(dest)?;
        Ok(())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a mock runtime context for testing
pub fn mock_context() -> RuntimeContext {
    let fs = Arc::new(MockFileSystem::new());
    RuntimeContext {
        paths: Arc::new(MockPathProvider::new("/tmp/vx-test")),
        http: Arc::new(MockHttpClient::new()),
        fs: fs.clone(),
        installer: Arc::new(MockInstaller::with_fs(fs)),
        config: RuntimeConfig::default(),
        version_cache: None,
        download_url_cache: None,
    }
}

/// Create a mock execution context for testing
pub fn mock_execution_context() -> ExecutionContext {
    ExecutionContext::new(Arc::new(MockCommandExecutor::new()))
}

// ============================================================================
// Runtime Tester (RFC 0020)
// ============================================================================

use regex::Regex;
use std::process::Command;
use std::time::{Duration, Instant};
use vx_manifest::{TestCommand, TestConfig};

/// Result of testing a runtime
#[derive(Debug, Clone)]
pub struct RuntimeTestResult {
    /// Runtime name
    pub runtime_name: String,
    /// Whether the platform is supported
    pub platform_supported: bool,
    /// Whether the runtime is installed in vx store
    pub installed: bool,
    /// Whether the runtime is available in system PATH
    pub system_available: bool,
    /// Individual test case results
    pub test_cases: Vec<TestCaseResult>,
    /// Overall pass/fail status
    pub overall_passed: bool,
    /// Total duration of all tests
    pub total_duration: Duration,
    /// Error message if testing failed early
    pub error: Option<String>,
}

impl RuntimeTestResult {
    /// Create a new test result for a runtime
    pub fn new(runtime_name: impl Into<String>) -> Self {
        Self {
            runtime_name: runtime_name.into(),
            platform_supported: true,
            installed: false,
            system_available: false,
            test_cases: Vec::new(),
            overall_passed: false,
            total_duration: Duration::ZERO,
            error: None,
        }
    }

    /// Mark as platform not supported
    pub fn platform_not_supported(mut self) -> Self {
        self.platform_supported = false;
        self.overall_passed = false;
        self
    }

    /// Mark with an error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self.overall_passed = false;
        self
    }

    /// Calculate overall status from test cases
    /// Note: This is a low-level finalization. The actual pass/fail logic
    /// depends on the test mode (config check vs functional test) and is
    /// handled by the CLI handler.
    pub fn finalize(mut self) -> Self {
        // For RuntimeTestResult, overall_passed means:
        // - Platform is supported
        // - No errors occurred
        // - If tests were run, they all passed
        // The availability check (installed || system_available) is handled by the CLI
        self.overall_passed = self.platform_supported
            && self.error.is_none()
            && self.test_cases.iter().all(|t| t.passed);
        self
    }

    /// Get the number of passed tests
    pub fn passed_count(&self) -> usize {
        self.test_cases.iter().filter(|t| t.passed).count()
    }

    /// Get the number of failed tests
    pub fn failed_count(&self) -> usize {
        self.test_cases.iter().filter(|t| !t.passed).count()
    }
}

/// Result of a single test case
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestCaseResult {
    /// Test name/description
    pub name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Command output (stdout)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    /// Command error output (stderr)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    /// Exit code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Error message if test failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Test duration in milliseconds
    #[serde(serialize_with = "serialize_duration")]
    pub duration: Duration,
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(duration.as_secs_f64() * 1000.0)
}

impl TestCaseResult {
    /// Create a passed test result
    pub fn passed(name: impl Into<String>, duration: Duration) -> Self {
        Self {
            name: name.into(),
            passed: true,
            stdout: None,
            stderr: None,
            exit_code: Some(0),
            error: None,
            duration,
        }
    }

    /// Create a failed test result
    pub fn failed(name: impl Into<String>, error: impl Into<String>, duration: Duration) -> Self {
        Self {
            name: name.into(),
            passed: false,
            stdout: None,
            stderr: None,
            exit_code: None,
            error: Some(error.into()),
            duration,
        }
    }

    /// Set output
    pub fn with_output(mut self, stdout: String, stderr: String, exit_code: i32) -> Self {
        self.stdout = Some(stdout);
        self.stderr = Some(stderr);
        self.exit_code = Some(exit_code);
        self
    }
}

/// Runtime tester that uses manifest-based test configuration
pub struct RuntimeTester {
    /// Runtime name
    runtime_name: String,
    /// Executable path (if installed)
    executable_path: Option<PathBuf>,
    /// Test configuration from manifest
    test_config: Option<TestConfig>,
    /// Timeout for tests
    timeout: Duration,
}

impl RuntimeTester {
    /// Create a new runtime tester
    pub fn new(runtime_name: impl Into<String>) -> Self {
        Self {
            runtime_name: runtime_name.into(),
            executable_path: None,
            test_config: None,
            timeout: Duration::from_secs(30),
        }
    }

    /// Set the executable path
    pub fn with_executable(mut self, path: PathBuf) -> Self {
        self.executable_path = Some(path);
        self
    }

    /// Set the test configuration
    pub fn with_config(mut self, config: TestConfig) -> Self {
        self.timeout = Duration::from_millis(config.timeout_ms);
        self.test_config = Some(config);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Run all tests
    pub fn run_all(&self) -> RuntimeTestResult {
        let start = Instant::now();
        let mut result = RuntimeTestResult::new(&self.runtime_name);

        // Check availability first
        result.installed = self.executable_path.as_ref().is_some_and(|p| p.exists());
        result.system_available = self.check_system_available();

        // Check if we have an executable
        let executable = match self.get_executable() {
            Some(exe) => exe,
            None => {
                // No executable found - this is not an error for configuration checks
                // Just return with installed=false, system_available=false
                result.total_duration = start.elapsed();
                return result.finalize();
            }
        };

        // Get test commands
        let test_commands = self.get_test_commands();

        // Run each test command
        for cmd in test_commands {
            let test_result = self.run_test_command(&cmd, &executable);
            result.test_cases.push(test_result);
        }

        // Run inline script if configured
        if let Some(script_result) = self.run_inline_script(&executable) {
            result.test_cases.push(script_result);
        }

        // If no tests were configured, run default version test
        if result.test_cases.is_empty() {
            let default_result = self.run_default_test(&executable);
            result.test_cases.push(default_result);
        }

        result.total_duration = start.elapsed();
        result.finalize()
    }

    /// Get the executable to use for testing
    fn get_executable(&self) -> Option<String> {
        if let Some(ref path) = self.executable_path {
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }

        // Fall back to system PATH
        if self.check_system_available() {
            return Some(self.runtime_name.clone());
        }

        None
    }

    /// Check if runtime is available in system PATH
    fn check_system_available(&self) -> bool {
        which::which(&self.runtime_name).is_ok()
    }

    /// Get test commands from config or defaults
    fn get_test_commands(&self) -> Vec<TestCommand> {
        if let Some(ref config) = self.test_config {
            let mut commands = config.functional_commands.clone();

            // Add platform-specific commands
            if let Some(ref platforms) = config.platforms {
                #[cfg(windows)]
                if let Some(ref win) = platforms.windows {
                    commands.extend(win.functional_commands.clone());
                }

                #[cfg(unix)]
                if let Some(ref unix) = platforms.unix {
                    commands.extend(unix.functional_commands.clone());
                }
            }

            commands
        } else {
            Vec::new()
        }
    }

    /// Run a single test command
    fn run_test_command(&self, cmd: &TestCommand, executable: &str) -> TestCaseResult {
        let start = Instant::now();
        let test_name = cmd.display_name().to_string();

        // Substitute variables in command
        let command_str = cmd.command.replace("{executable}", executable);

        // Parse command with proper quote handling
        let parts = match parse_command_line(&command_str) {
            Ok(parts) => parts,
            Err(e) => {
                return TestCaseResult::failed(
                    &test_name,
                    format!("Failed to parse command: {}", e),
                    start.elapsed(),
                );
            }
        };

        if parts.is_empty() {
            return TestCaseResult::failed(&test_name, "Empty command", start.elapsed());
        }

        let program = &parts[0];
        let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

        // Execute command with timeout
        let output = match run_command_with_timeout(program, &args, self.timeout) {
            Ok(output) => output,
            Err(e) => {
                return TestCaseResult::failed(
                    &test_name,
                    format!("Failed to execute: {}", e),
                    start.elapsed(),
                );
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let duration = start.elapsed();

        // Check expectations
        let mut passed = true;
        let mut error_msg = None;

        // Check exit code
        if let Some(expected_code) = cmd.expected_exit_code {
            if exit_code != expected_code {
                passed = false;
                error_msg = Some(format!(
                    "Expected exit code {}, got {}",
                    expected_code, exit_code
                ));
            }
        } else if cmd.expect_success && exit_code != 0 {
            passed = false;
            error_msg = Some(format!("Expected success, got exit code {}", exit_code));
        }

        // Check output pattern
        if passed {
            if let Some(ref pattern) = cmd.expected_output {
                match Regex::new(pattern) {
                    Ok(re) => {
                        if !re.is_match(&stdout) && !re.is_match(&stderr) {
                            passed = false;
                            error_msg = Some(format!(
                                "Output did not match pattern '{}'\nstdout: {}\nstderr: {}",
                                pattern, stdout, stderr
                            ));
                        }
                    }
                    Err(e) => {
                        passed = false;
                        error_msg = Some(format!("Invalid regex pattern '{}': {}", pattern, e));
                    }
                }
            }
        }

        TestCaseResult {
            name: test_name,
            passed,
            stdout: Some(stdout),
            stderr: Some(stderr),
            exit_code: Some(exit_code),
            error: error_msg,
            duration,
        }
    }

    /// Run inline script if configured
    fn run_inline_script(&self, executable: &str) -> Option<TestCaseResult> {
        let scripts = self.test_config.as_ref()?.inline_scripts.as_ref()?;
        let script = scripts.for_current_platform()?;

        let start = Instant::now();
        let test_name = "inline_script";

        // Substitute variables
        let script_content = script.replace("{executable}", executable);

        // Create temp script file and execute
        let result = self.execute_inline_script(&script_content);

        let duration = start.elapsed();

        match result {
            Ok((stdout, stderr, exit_code)) => {
                let passed = exit_code == 0;
                Some(TestCaseResult {
                    name: test_name.to_string(),
                    passed,
                    stdout: Some(stdout),
                    stderr: Some(stderr),
                    exit_code: Some(exit_code),
                    error: if passed {
                        None
                    } else {
                        Some(format!("Script exited with code {}", exit_code))
                    },
                    duration,
                })
            }
            Err(e) => Some(TestCaseResult::failed(
                test_name,
                format!("Failed to execute script: {}", e),
                duration,
            )),
        }
    }

    /// Execute an inline script
    fn execute_inline_script(&self, script: &str) -> Result<(String, String, i32)> {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();

        #[cfg(windows)]
        let script_path = temp_dir.join(format!("vx_test_{}.bat", std::process::id()));
        #[cfg(unix)]
        let script_path = temp_dir.join(format!("vx_test_{}.sh", std::process::id()));

        // Write script to temp file
        let mut file = std::fs::File::create(&script_path)?;
        file.write_all(script.as_bytes())?;
        drop(file);

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;
        }

        // Execute script with timeout
        #[cfg(windows)]
        let output =
            run_command_with_timeout("cmd", &["/C", &script_path.to_string_lossy()], self.timeout)?;

        #[cfg(unix)]
        let output =
            run_command_with_timeout("sh", &[&script_path.to_string_lossy()], self.timeout)?;

        // Clean up
        let _ = std::fs::remove_file(&script_path);

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        Ok((stdout, stderr, exit_code))
    }

    /// Run default version test
    fn run_default_test(&self, executable: &str) -> TestCaseResult {
        let start = Instant::now();
        let test_name = "version_check";

        let output = match run_command_with_timeout(executable, &["--version"], self.timeout) {
            Ok(output) => output,
            Err(e) => {
                return TestCaseResult::failed(
                    test_name,
                    format!("Failed to execute: {}", e),
                    start.elapsed(),
                );
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let duration = start.elapsed();

        // Consider it passed if we got any output (some tools output version to stderr)
        let passed = exit_code == 0 || !stdout.is_empty() || !stderr.is_empty();

        TestCaseResult {
            name: test_name.to_string(),
            passed,
            stdout: Some(stdout),
            stderr: Some(stderr),
            exit_code: Some(exit_code),
            error: if passed {
                None
            } else {
                Some("No version output".to_string())
            },
            duration,
        }
    }
}

/// Run a command with timeout support
///
/// This function spawns a child process and waits for it to complete with a timeout.
/// If the timeout is exceeded, the process is killed and an error is returned.
fn run_command_with_timeout(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> std::io::Result<std::process::Output> {
    use std::io::Read;

    let mut child = Command::new(program)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null()) // Prevent waiting for input
        .spawn()?;

    let start = Instant::now();

    // Poll for completion with timeout
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has exited
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();

                if let Some(mut stdout_pipe) = child.stdout.take() {
                    let _ = stdout_pipe.read_to_end(&mut stdout);
                }
                if let Some(mut stderr_pipe) = child.stderr.take() {
                    let _ = stderr_pipe.read_to_end(&mut stderr);
                }

                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                // Process still running
                if start.elapsed() > timeout {
                    // Timeout exceeded - kill the process
                    let _ = child.kill();
                    let _ = child.wait(); // Clean up zombie process
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!("Command timed out after {:?}", timeout),
                    ));
                }
                // Sleep briefly before polling again
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                // Error checking status
                let _ = child.kill();
                return Err(e);
            }
        }
    }
}

/// Parse a command line string into arguments, respecting quotes
///
/// Examples:
/// - `node --version` -> ["node", "--version"]
/// - `node -e "console.log('hello')"` -> ["node", "-e", "console.log('hello')"]
/// - `C:\path\to\node.exe --version` -> ["C:\path\to\node.exe", "--version"]
fn parse_command_line(cmd: &str) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for c in cmd.chars() {
        match c {
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(current);
                    current = String::new();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    if in_single_quote {
        anyhow::bail!("Unclosed single quote");
    }
    if in_double_quote {
        anyhow::bail!("Unclosed double quote");
    }

    Ok(args)
}
