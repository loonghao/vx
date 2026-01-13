//! Abstract traits for dependency injection
//!
//! These traits allow mocking external dependencies for testing.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// HTTP client abstraction for testability
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Perform a GET request and return the response body as string
    async fn get(&self, url: &str) -> Result<String>;

    /// Perform a GET request and return the response body as JSON Value
    async fn get_json_value(&self, url: &str) -> Result<serde_json::Value>;

    /// Download a file to the specified path
    async fn download(&self, url: &str, dest: &Path) -> Result<()>;

    /// Download a file with progress callback (total_bytes, downloaded_bytes)
    async fn download_with_progress(
        &self,
        url: &str,
        dest: &Path,
        on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    ) -> Result<()>;

    /// Download a file with caching support
    ///
    /// If the file is already cached, it will be copied from cache.
    /// Otherwise, downloads and stores in cache for future use.
    ///
    /// Returns true if served from cache, false if downloaded.
    async fn download_cached(&self, url: &str, dest: &Path) -> Result<bool> {
        // Default implementation: just download without caching
        self.download(url, dest).await?;
        Ok(false)
    }

    /// Check if a URL is cached
    fn is_cached(&self, _url: &str) -> bool {
        false
    }
}

/// File system abstraction for testability
pub trait FileSystem: Send + Sync {
    /// Check if a path exists
    fn exists(&self, path: &Path) -> bool;

    /// Check if a path is a directory
    fn is_dir(&self, path: &Path) -> bool;

    /// Check if a path is a file
    fn is_file(&self, path: &Path) -> bool;

    /// Create a directory and all parent directories
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Remove a directory and all its contents
    fn remove_dir_all(&self, path: &Path) -> Result<()>;

    /// Remove a file
    fn remove_file(&self, path: &Path) -> Result<()>;

    /// Read directory contents
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;

    /// Read file contents as string
    fn read_to_string(&self, path: &Path) -> Result<String>;

    /// Read file contents as bytes
    fn read(&self, path: &Path) -> Result<Vec<u8>>;

    /// Write string to file
    fn write(&self, path: &Path, content: &str) -> Result<()>;

    /// Write bytes to file
    fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<()>;

    /// Copy a file
    fn copy(&self, from: &Path, to: &Path) -> Result<()>;

    /// Create a hard link
    fn hard_link(&self, src: &Path, dst: &Path) -> Result<()>;

    /// Create a symbolic link
    fn symlink(&self, src: &Path, dst: &Path) -> Result<()>;

    /// Set file permissions (Unix only)
    #[cfg(unix)]
    fn set_permissions(&self, path: &Path, mode: u32) -> Result<()>;
}

/// Command executor abstraction for testability
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Execute a command and return the result
    async fn execute(
        &self,
        program: &str,
        args: &[String],
        working_dir: Option<&Path>,
        env: &HashMap<String, String>,
        capture_output: bool,
    ) -> Result<crate::types::ExecutionResult>;

    /// Check if a program exists in PATH
    fn which(&self, program: &str) -> Option<PathBuf>;
}

/// Path provider abstraction for testability
pub trait PathProvider: Send + Sync {
    /// Get the VX home directory (~/.vx)
    fn vx_home(&self) -> PathBuf;

    /// Get the store directory (~/.vx/store)
    fn store_dir(&self) -> PathBuf;

    /// Get the environments directory (~/.vx/envs)
    fn envs_dir(&self) -> PathBuf;

    /// Get the bin directory (~/.vx/bin)
    fn bin_dir(&self) -> PathBuf;

    /// Get the cache directory (~/.vx/cache)
    fn cache_dir(&self) -> PathBuf;

    /// Get the config directory (~/.vx/config)
    fn config_dir(&self) -> PathBuf;

    /// Get the directory for a specific runtime in the store
    fn runtime_store_dir(&self, name: &str) -> PathBuf;

    /// Get the directory for a specific version of a runtime in the store
    fn version_store_dir(&self, name: &str, version: &str) -> PathBuf;

    /// Get the executable path for a specific version of a runtime
    fn executable_path(&self, name: &str, version: &str) -> PathBuf;

    /// Get the environment directory
    fn env_dir(&self, env_name: &str) -> PathBuf;

    // ========== npm-tools paths ==========

    /// Get the npm-tools directory (~/.vx/npm-tools)
    fn npm_tools_dir(&self) -> PathBuf;

    /// Get the npm-tools directory for a specific package
    fn npm_tool_dir(&self, package_name: &str) -> PathBuf;

    /// Get the npm-tools directory for a specific package version
    fn npm_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf;

    /// Get the bin directory for an npm tool
    fn npm_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf;

    // ========== pip-tools paths ==========

    /// Get the pip-tools directory (~/.vx/pip-tools)
    fn pip_tools_dir(&self) -> PathBuf;

    /// Get the pip-tools directory for a specific package
    fn pip_tool_dir(&self, package_name: &str) -> PathBuf;

    /// Get the pip-tools directory for a specific package version
    fn pip_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf;

    /// Get the venv directory for a pip tool
    fn pip_tool_venv_dir(&self, package_name: &str, version: &str) -> PathBuf;

    /// Get the bin directory for a pip tool
    fn pip_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf;
}

/// Installer abstraction for testability
#[async_trait]
pub trait Installer: Send + Sync {
    /// Extract an archive to a directory
    async fn extract(&self, archive: &Path, dest: &Path) -> Result<()>;

    /// Download and extract in one operation
    async fn download_and_extract(&self, url: &str, dest: &Path) -> Result<()>;

    /// Download and install with layout configuration (RFC 0019)
    ///
    /// This method accepts layout metadata to handle file renaming, moving, and permissions.
    /// If not implemented, falls back to `download_and_extract`.
    async fn download_with_layout(
        &self,
        url: &str,
        dest: &Path,
        metadata: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        // Default implementation - use metadata for post-processing
        self.download_and_extract(url, dest).await?;

        // Apply layout transformations if metadata is provided
        if let (Some(source_name), Some(target_name), Some(target_dir)) = (
            metadata.get("source_name"),
            metadata.get("target_name"),
            metadata.get("target_dir"),
        ) {
            let source_path = dest.join(target_dir).join(source_name);
            let target_path = dest.join(target_dir).join(target_name);

            if source_path.exists() && source_path != target_path {
                // On Windows, rename might fail if target exists, so remove target first
                if target_path.exists() {
                    let _ = std::fs::remove_file(&target_path);
                }

                // Try rename first (atomic on same filesystem)
                if std::fs::rename(&source_path, &target_path).is_err() {
                    // Fallback to copy + delete
                    std::fs::copy(&source_path, &target_path)?;
                    let _ = std::fs::remove_file(&source_path);
                }

                // Set permissions if specified
                #[cfg(unix)]
                if let Some(perm_str) = metadata.get("target_permissions") {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(mode) = u32::from_str_radix(perm_str, 8) {
                        let mut perms = std::fs::metadata(&target_path)?.permissions();
                        perms.set_mode(mode);
                        std::fs::set_permissions(&target_path, perms)?;
                    }
                }
            }
        }

        Ok(())
    }
}
