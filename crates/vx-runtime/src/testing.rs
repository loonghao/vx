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
    }
}

/// Create a mock execution context for testing
pub fn mock_execution_context() -> ExecutionContext {
    ExecutionContext::new(Arc::new(MockCommandExecutor::new()))
}
