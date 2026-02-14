//! Path manager for vx tool installations

use crate::{VxPaths, with_executable_extension};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Current platform information
///
/// This is a lightweight platform detection used within vx-paths
/// to avoid circular dependency with vx-runtime.
#[derive(Debug, Clone, Copy)]
pub struct CurrentPlatform {
    /// Operating system
    pub os: &'static str,
    /// Architecture
    pub arch: &'static str,
}

impl CurrentPlatform {
    /// Detect current platform
    pub fn current() -> Self {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "freebsd") {
            "freebsd"
        } else {
            "unknown"
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else if cfg!(target_arch = "arm") {
            "arm"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            "unknown"
        };

        Self { os, arch }
    }

    /// Get platform string for directory names
    ///
    /// Returns strings like "windows-x64", "darwin-arm64", "linux-x64", etc.
    pub fn as_str(&self) -> String {
        format!("{}-{}", self.os, self.arch)
    }
}

/// Manages paths for vx tool installations with standardized structure
///
/// The PathManager provides platform-agnostic access to the vx store.
/// All access to `<provider>/<version>/` automatically redirects to
/// `<provider>/<version>/<platform>/` for the current platform.
///
/// # Platform Redirection
///
/// - **External API**: Uses `<provider>/<version>/` paths
/// - **Internal Storage**: Uses `<provider>/<version>/<platform>/` paths
/// - **Automatic Redirection**: The PathManager handles platform redirection transparently
///
/// # Directory Structure
///
/// ```text
/// ~/.vx/store/
/// ├── node/
/// │   └── 20.0.0/              # Unified version directory
/// │       ├── windows-x64/          # Platform-specific (internal)
/// │       ├── darwin-x64/
/// │       └── linux-x64/
/// └── python/
///     └── 3.9.21/
///         ├── windows-x64/
///         └── linux-x64/
/// ```
#[derive(Debug, Clone)]
pub struct PathManager {
    paths: VxPaths,
}

impl PathManager {
    /// Create a new PathManager with default paths
    pub fn new() -> Result<Self> {
        let paths = VxPaths::new()?;
        paths.ensure_dirs()?;
        Ok(Self { paths })
    }

    /// Create a new PathManager with custom base directory
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        let paths = VxPaths::with_base_dir(base_dir);
        paths.ensure_dirs()?;
        Ok(Self { paths })
    }

    /// Create a PathManager from existing VxPaths
    pub fn from_paths(paths: VxPaths) -> Self {
        Self { paths }
    }

    // ========== Base Directories ==========

    /// Get the base vx directory
    pub fn base_dir(&self) -> &Path {
        &self.paths.base_dir
    }

    /// Get the global store directory
    pub fn store_dir(&self) -> &Path {
        &self.paths.store_dir
    }

    /// Get the environments directory
    pub fn envs_dir(&self) -> &Path {
        &self.paths.envs_dir
    }

    /// Get the bin directory (for shims)
    pub fn bin_dir(&self) -> &Path {
        &self.paths.bin_dir
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.paths.cache_dir
    }

    /// Get the config directory
    pub fn config_dir(&self) -> &Path {
        &self.paths.config_dir
    }

    /// Get the temporary directory
    pub fn tmp_dir(&self) -> &Path {
        &self.paths.tmp_dir
    }

    // ========== Store Paths (Content-Addressable Storage) ==========

    /// Get the platform directory name for the current platform
    ///
    /// Returns platform string like "windows-x64", "darwin-arm64", "linux-x64", etc.
    ///
    /// This is used internally for platform-specific storage.
    pub fn platform_dir_name(&self) -> String {
        CurrentPlatform::current().as_str()
    }

    /// Get the actual platform-specific store directory for a runtime version
    ///
    /// Returns: ~/.vx/store/<runtime>/<version>/<platform>
    ///
    /// This is the **actual** directory where files are stored.
    /// Use this when installing or directly accessing platform-specific files.
    ///
    /// # Example
    /// ```ignore
    /// let manager = PathManager::new()?;
    /// let platform_dir = manager.platform_store_dir("python", "3.9.21");
    /// // Returns: ~/.vx/store/python/3.9.21/windows-x64 (on Windows x64)
    /// ```
    pub fn platform_store_dir(&self, runtime_name: &str, version: &str) -> PathBuf {
        self.version_store_dir(runtime_name, version)
            .join(self.platform_dir_name())
    }

    /// Get the store directory for a specific runtime
    /// Returns: ~/.vx/store/<runtime>
    ///
    /// This returns the unified runtime directory, not platform-specific.
    pub fn runtime_store_dir(&self, runtime_name: &str) -> PathBuf {
        self.paths.store_dir.join(runtime_name)
    }

    /// Get the store directory for a specific runtime version
    /// Returns: ~/.vx/store/<runtime>/<version>
    ///
    /// This returns the unified version directory. All file access through this
    /// path will automatically redirect to the platform-specific directory.
    ///
    /// # Platform Redirection
    /// When checking if a version exists or accessing files, the PathManager
    /// automatically redirects to `<runtime>/<version>/<platform>/` for
    /// the current platform.
    pub fn version_store_dir(&self, runtime_name: &str, version: &str) -> PathBuf {
        self.runtime_store_dir(runtime_name).join(version)
    }

    /// Get the actual executable path in the platform-specific store
    ///
    /// Returns: ~/.vx/store/<runtime>/<version>/<platform>/bin/<runtime>.exe (Windows)
    ///
    /// This returns the path to the executable in the platform-specific directory.
    /// Use this when installing or directly accessing executables.
    ///
    /// # Example
    /// ```ignore
    /// let manager = PathManager::new()?;
    /// let exe_path = manager.platform_executable_path("python", "3.9.21");
    /// // Returns: ~/.vx/store/python/3.9.21/windows-x64/python.exe (on Windows)
    /// ```
    pub fn platform_executable_path(&self, runtime_name: &str, version: &str) -> PathBuf {
        let platform_dir = self.platform_store_dir(runtime_name, version);
        let executable_name = with_executable_extension(runtime_name);
        platform_dir.join("bin").join(executable_name)
    }

    /// Get the executable path in the store for a specific runtime version
    /// Returns: ~/.vx/store/<runtime>/<version>/bin/<runtime>.exe (Windows)
    ///
    /// This is a **unified** path that automatically redirects to the
    /// platform-specific directory. Use this for general executable access.
    ///
    /// # Note
    /// For actual file operations (install, check existence), use `platform_executable_path()`.
    pub fn store_executable_path(&self, runtime_name: &str, version: &str) -> PathBuf {
        let version_dir = self.version_store_dir(runtime_name, version);
        let executable_name = with_executable_extension(runtime_name);
        version_dir.join("bin").join(executable_name)
    }

    /// Check if a runtime version is installed in the store
    ///
    /// This checks the platform-specific directory:
    /// `~/.vx/store/<runtime>/<version>/<platform>/`
    pub fn is_version_in_store(&self, runtime_name: &str, version: &str) -> bool {
        let platform_dir = self.platform_store_dir(runtime_name, version);
        platform_dir.exists()
    }

    /// List all installed versions of a runtime in the store
    ///
    /// This checks the new directory structure:
    /// - New: <runtime>/<version>/<platform>/
    ///
    /// Returns: List of version strings, sorted by semantic version (highest first)
    pub fn list_store_versions(&self, runtime_name: &str) -> Result<Vec<String>> {
        let runtime_dir = self.runtime_store_dir(runtime_name);

        if !runtime_dir.exists() {
            return Ok(Vec::new());
        }

        let current_platform = self.platform_dir_name();
        let mut versions = Vec::new();

        // Scan version directories
        for entry in std::fs::read_dir(&runtime_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only check directories
            if !entry.file_type()?.is_dir() {
                continue;
            }

            // Check if this is a version directory (e.g., "3.13.4")
            // Version directories should start with a digit
            let version_str = entry.file_name().to_string_lossy().to_string();

            // Skip non-version directories
            if !version_str
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                continue;
            }

            // Check new structure: <version>/<platform>/
            let platform_dir = path.join(&current_platform);
            if platform_dir.exists() {
                versions.push(version_str);
            }
        }

        // Sort by semantic version (highest first)
        versions.sort_by(|a, b| {
            semver::Version::parse(a)
                .and_then(|va| semver::Version::parse(b).map(|vb| vb.cmp(&va)))
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse() // Highest first
        });
        Ok(versions)
    }

    /// List all runtimes in the store
    pub fn list_store_runtimes(&self) -> Result<Vec<String>> {
        if !self.paths.store_dir.exists() {
            return Ok(Vec::new());
        }

        let mut runtimes = Vec::new();
        for entry in std::fs::read_dir(&self.paths.store_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                runtimes.push(name.to_string());
            }
        }

        runtimes.sort();
        Ok(runtimes)
    }

    // ========== Environment Paths ==========

    /// Get the directory for a specific environment
    /// Returns: ~/.vx/envs/<env_name>
    pub fn env_dir(&self, env_name: &str) -> PathBuf {
        self.paths.envs_dir.join(env_name)
    }

    /// Get the default environment directory
    /// Returns: ~/.vx/envs/default
    pub fn default_env_dir(&self) -> PathBuf {
        self.paths.envs_dir.join("default")
    }

    /// Get the runtime link path in an environment
    /// Returns: ~/.vx/envs/<env_name>/<runtime>
    pub fn env_runtime_path(&self, env_name: &str, runtime_name: &str) -> PathBuf {
        self.env_dir(env_name).join(runtime_name)
    }

    /// List all environments
    pub fn list_envs(&self) -> Result<Vec<String>> {
        if !self.paths.envs_dir.exists() {
            return Ok(Vec::new());
        }

        let mut envs = Vec::new();
        for entry in std::fs::read_dir(&self.paths.envs_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                envs.push(name.to_string());
            }
        }

        envs.sort();
        Ok(envs)
    }

    /// Check if an environment exists
    pub fn env_exists(&self, env_name: &str) -> bool {
        self.env_dir(env_name).exists()
    }

    /// Create an environment directory
    pub fn create_env(&self, env_name: &str) -> Result<PathBuf> {
        let env_dir = self.env_dir(env_name);
        std::fs::create_dir_all(&env_dir)?;
        Ok(env_dir)
    }

    /// Remove an environment
    pub fn remove_env(&self, env_name: &str) -> Result<()> {
        let env_dir = self.env_dir(env_name);
        if env_dir.exists() {
            std::fs::remove_dir_all(&env_dir)?;
        }
        Ok(())
    }

    // ========== Cache and Temp Paths ==========

    /// Get cache path for a tool
    pub fn tool_cache_dir(&self, tool_name: &str) -> PathBuf {
        self.paths.cache_dir.join(tool_name)
    }

    /// Get temporary path for a tool installation
    pub fn tool_tmp_dir(&self, tool_name: &str, version: &str) -> PathBuf {
        self.paths
            .tmp_dir
            .join(format!("{}-{}", tool_name, version))
    }

    // ========== npm-tools Paths ==========

    /// Get the npm-tools directory
    pub fn npm_tools_dir(&self) -> &Path {
        &self.paths.npm_tools_dir
    }

    /// Get the npm-tools directory for a specific package
    /// Returns: ~/.vx/npm-tools/<package>
    pub fn npm_tool_dir(&self, package_name: &str) -> PathBuf {
        self.paths.npm_tools_dir.join(package_name)
    }

    /// Get the npm-tools directory for a specific package version
    /// Returns: ~/.vx/npm-tools/<package>/<version>
    pub fn npm_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.npm_tool_dir(package_name).join(version)
    }

    /// Get the bin directory for an npm tool
    /// Returns: ~/.vx/npm-tools/<package>/<version>/bin
    pub fn npm_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.npm_tool_version_dir(package_name, version).join("bin")
    }

    /// List all installed versions of an npm tool
    pub fn list_npm_tool_versions(&self, package_name: &str) -> Result<Vec<String>> {
        let tool_dir = self.npm_tool_dir(package_name);
        if !tool_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        for entry in std::fs::read_dir(&tool_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && let Some(version) = entry.file_name().to_str()
            {
                versions.push(version.to_string());
            }
        }

        versions.sort();
        Ok(versions)
    }

    // ========== pip-tools Paths ==========

    /// Get the pip-tools directory
    pub fn pip_tools_dir(&self) -> &Path {
        &self.paths.pip_tools_dir
    }

    /// Get the pip-tools directory for a specific package
    /// Returns: ~/.vx/pip-tools/<package>
    pub fn pip_tool_dir(&self, package_name: &str) -> PathBuf {
        self.paths.pip_tools_dir.join(package_name)
    }

    /// Get the pip-tools directory for a specific package version
    /// Returns: ~/.vx/pip-tools/<package>/<version>
    pub fn pip_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.pip_tool_dir(package_name).join(version)
    }

    /// Get the venv directory for a pip tool
    /// Returns: ~/.vx/pip-tools/<package>/<version>/venv
    pub fn pip_tool_venv_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.pip_tool_version_dir(package_name, version)
            .join("venv")
    }

    /// Get the bin directory for a pip tool
    /// Returns: ~/.vx/pip-tools/<package>/<version>/venv/Scripts (Windows) or venv/bin (Unix)
    pub fn pip_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        let venv_dir = self.pip_tool_venv_dir(package_name, version);
        if cfg!(windows) {
            venv_dir.join("Scripts")
        } else {
            venv_dir.join("bin")
        }
    }

    /// List all installed versions of a pip tool
    pub fn list_pip_tool_versions(&self, package_name: &str) -> Result<Vec<String>> {
        let tool_dir = self.pip_tool_dir(package_name);
        if !tool_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        for entry in std::fs::read_dir(&tool_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && let Some(version) = entry.file_name().to_str()
            {
                versions.push(version.to_string());
            }
        }

        versions.sort();
        Ok(versions)
    }
}

impl Default for PathManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self::with_base_dir(".vx")
                .expect("Failed to create PathManager with fallback directory")
        })
    }
}
