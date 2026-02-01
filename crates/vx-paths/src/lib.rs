//! Cross-platform path management for vx tool installations
//!
//! This crate provides a unified interface for managing tool installation paths
//! across different platforms, ensuring consistent directory structures and
//! proper handling of executable file extensions.
//!
//! # Platform Redirection
//!
//! vx uses a **platform-agnostic API** with automatic platform-specific storage.
//!
//! - **External API**: Access tools using `<provider>/<version>/` paths
//! - **Internal Storage**: Files stored in `<provider>/<version>/<platform>/` directories
//! - **Automatic Redirection**: PathManager transparently redirects to current platform
//!
//! # Directory Structure
//!
//! ```text
//! ~/.vx/
//! ├── store/                      # Global storage (Content-Addressable)
//! │   ├── node/
//! │   │   └── 20.0.0/           # Unified version directory (API)
//! │   │       ├── windows-x64/      # Platform-specific (storage)
//! │   │       ├── darwin-x64/
//! │   │       └── linux-x64/
//! │   ├── go/
//! │   │   └── 1.21.0/
//! │   │       ├── windows-x64/
//! │   │       └── linux-x64/
//! │   └── python/
//! │       └── 3.9.21/
//! │           ├── windows-x64/
//! │           └── linux-x64/
//! │
//! ├── npm-tools/                  # npm package tools (isolated environments)
//! │   └── vite/
//! │       └── 5.4.0/
//! │           ├── node_modules/
//! │           └── bin/vite        # shim script
//! │
//! ├── pip-tools/                  # pip package tools (isolated environments)
//! │   └── rez/
//! │       └── 2.114.0/
//! │           ├── venv/
//! │           └── bin/rez         # shim script
//! │
//! ├── envs/                       # Virtual environments (links to store)
//! │   ├── default/               # Default environment
//! │   │   └── node -> ../../store/node/20.0.0
//! │   └── project-abc/           # Project-specific environment
//! │       └── node -> ../../store/node/18.0.0
//! │
//! ├── providers/                  # User-defined manifest-driven providers
//! │   ├── unix-tools/            # Example: Unix philosophy tools
//! │   │   └── provider.toml
//! │   └── my-custom-tools/       # User's custom tools
//! │       └── provider.toml
//! │
//! ├── bin/                        # Global shims
//! ├── cache/                      # Download cache
//! ├── config/                     # Configuration
//! └── tmp/                        # Temporary files
//! ```
//!
//! # Offline Bundle Support
//!
//! The platform redirection design enables efficient offline bundles:
//!
//! ```text
//! bundle/
//! └── store/
//!     └── node/
//!         └── 20.0.0/
//!             ├── windows-x64/    # All platforms in one bundle
//!             ├── darwin-x64/
//!             └── linux-x64/
//! ```
//!
//! When extracting the bundle, vx automatically selects the correct platform
//! directory for the current system.

use anyhow::Result;
use std::path::{Path, PathBuf};

pub mod config;
pub mod global_packages;
pub mod link;
pub mod manager;
pub mod package_spec;
pub mod platform;
pub mod project;
pub mod resolver;
pub mod shims;
pub mod windows;

pub use config::PathConfig;
pub use global_packages::{GlobalPackage, PackageRegistry, RuntimeDependency};
pub use link::{LinkResult, LinkStrategy};
pub use manager::PathManager;
pub use package_spec::PackageSpec;
pub use project::{
    find_config_file, find_config_file_upward, find_project_root, find_vx_config, is_in_vx_project,
    project_env_dir, ConfigNotFoundError, CONFIG_FILE_NAME, CONFIG_FILE_NAME_LEGACY, CONFIG_NAMES,
    LOCK_FILE_NAME, LOCK_FILE_NAMES, LOCK_FILE_NAME_LEGACY, PROJECT_BIN_DIR, PROJECT_CACHE_DIR,
    PROJECT_ENV_DIR, PROJECT_VX_DIR,
};
pub use resolver::{PathResolver, ToolLocation, ToolSource};

// Re-export platform module utilities for convenience
pub use platform::{
    append_to_path, executable_extension, filter_system_path, is_system_path, is_unix_path,
    is_windows_path, join_paths_env, join_paths_simple, path_separator, prepend_to_path,
    split_path, split_path_owned, venv_bin_dir, with_executable_extension, Arch, Os, Platform,
};

/// Standard vx directory structure
#[derive(Debug, Clone)]
pub struct VxPaths {
    /// Base vx directory (~/.vx)
    pub base_dir: PathBuf,
    /// Global store directory (~/.vx/store) - Content-Addressable Storage
    pub store_dir: PathBuf,
    /// npm package tools directory (~/.vx/npm-tools)
    pub npm_tools_dir: PathBuf,
    /// pip package tools directory (~/.vx/pip-tools)
    pub pip_tools_dir: PathBuf,
    /// Virtual environments directory (~/.vx/envs)
    pub envs_dir: PathBuf,
    /// Global shims directory (~/.vx/bin)
    pub bin_dir: PathBuf,
    /// Cache directory (~/.vx/cache)
    pub cache_dir: PathBuf,
    /// Configuration directory (~/.vx/config)
    pub config_dir: PathBuf,
    /// Temporary directory (~/.vx/tmp)
    pub tmp_dir: PathBuf,
    /// User providers directory (~/.vx/providers) - Manifest-driven runtimes
    pub providers_dir: PathBuf,
    /// Global packages CAS directory (~/.vx/packages) - RFC 0025
    pub packages_dir: PathBuf,
    /// Global shims directory (~/.vx/shims) - RFC 0025
    pub shims_dir: PathBuf,
}

impl VxPaths {
    /// Create VxPaths with default locations
    ///
    /// Uses VX_HOME environment variable if set, otherwise defaults to ~/.vx
    pub fn new() -> Result<Self> {
        // Check for VX_HOME environment variable first
        if let Ok(vx_home) = std::env::var("VX_HOME") {
            return Ok(Self::with_base_dir(vx_home));
        }

        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        let base_dir = home_dir.join(".vx");

        Ok(Self {
            store_dir: base_dir.join("store"),
            npm_tools_dir: base_dir.join("npm-tools"),
            pip_tools_dir: base_dir.join("pip-tools"),
            envs_dir: base_dir.join("envs"),
            bin_dir: base_dir.join("bin"),
            cache_dir: base_dir.join("cache"),
            config_dir: base_dir.join("config"),
            tmp_dir: base_dir.join("tmp"),
            providers_dir: base_dir.join("providers"),
            packages_dir: base_dir.join("packages"),
            shims_dir: base_dir.join("shims"),
            base_dir,
        })
    }

    /// Create VxPaths with custom base directory
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Self {
        let base_dir = base_dir.as_ref().to_path_buf();

        Self {
            store_dir: base_dir.join("store"),
            npm_tools_dir: base_dir.join("npm-tools"),
            pip_tools_dir: base_dir.join("pip-tools"),
            envs_dir: base_dir.join("envs"),
            bin_dir: base_dir.join("bin"),
            cache_dir: base_dir.join("cache"),
            config_dir: base_dir.join("config"),
            tmp_dir: base_dir.join("tmp"),
            providers_dir: base_dir.join("providers"),
            packages_dir: base_dir.join("packages"),
            shims_dir: base_dir.join("shims"),
            base_dir,
        }
    }

    /// Ensure all directories exist
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.base_dir)?;
        std::fs::create_dir_all(&self.store_dir)?;
        std::fs::create_dir_all(&self.npm_tools_dir)?;
        std::fs::create_dir_all(&self.pip_tools_dir)?;
        std::fs::create_dir_all(&self.envs_dir)?;
        std::fs::create_dir_all(&self.bin_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.tmp_dir)?;
        std::fs::create_dir_all(&self.providers_dir)?;
        std::fs::create_dir_all(&self.packages_dir)?;
        std::fs::create_dir_all(&self.shims_dir)?;
        Ok(())
    }

    /// Get the store directory for a specific runtime
    pub fn runtime_store_dir(&self, runtime_name: &str) -> PathBuf {
        self.store_dir.join(runtime_name)
    }

    /// Get the store directory for a specific runtime version
    pub fn version_store_dir(&self, runtime_name: &str, version: &str) -> PathBuf {
        self.runtime_store_dir(runtime_name).join(version)
    }

    /// Get the environment directory
    pub fn env_dir(&self, env_name: &str) -> PathBuf {
        self.envs_dir.join(env_name)
    }

    /// Get the default environment directory
    pub fn default_env_dir(&self) -> PathBuf {
        self.envs_dir.join("default")
    }

    // ========== npm-tools paths ==========

    /// Get the npm-tools directory for a specific package
    pub fn npm_tool_dir(&self, package_name: &str) -> PathBuf {
        self.npm_tools_dir.join(package_name)
    }

    /// Get the npm-tools directory for a specific package version
    pub fn npm_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.npm_tool_dir(package_name).join(version)
    }

    /// Get the bin directory for an npm tool
    pub fn npm_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.npm_tool_version_dir(package_name, version).join("bin")
    }

    // ========== pip-tools paths ==========

    /// Get the pip-tools directory for a specific package
    pub fn pip_tool_dir(&self, package_name: &str) -> PathBuf {
        self.pip_tools_dir.join(package_name)
    }

    /// Get the pip-tools directory for a specific package version
    pub fn pip_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.pip_tool_dir(package_name).join(version)
    }

    /// Get the venv directory for a pip tool
    pub fn pip_tool_venv_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.pip_tool_version_dir(package_name, version)
            .join("venv")
    }

    /// Get the bin directory for a pip tool
    pub fn pip_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        let venv_dir = self.pip_tool_venv_dir(package_name, version);
        venv_dir.join(venv_bin_dir())
    }

    // ========== RFC 0025: Global Packages CAS ==========

    /// Get the ecosystem directory for global packages
    ///
    /// Returns: ~/.vx/packages/{ecosystem}
    ///
    /// # Example
    /// ```
    /// use vx_paths::VxPaths;
    /// let paths = VxPaths::with_base_dir("/tmp/vx");
    /// let npm_dir = paths.ecosystem_packages_dir("npm");
    /// assert!(npm_dir.ends_with("packages/npm"));
    /// ```
    pub fn ecosystem_packages_dir(&self, ecosystem: &str) -> PathBuf {
        self.packages_dir.join(ecosystem.to_lowercase())
    }

    /// Get the package directory for a specific global package
    ///
    /// Returns: ~/.vx/packages/{ecosystem}/{package}/{version}
    ///
    /// # Example
    /// ```
    /// use vx_paths::VxPaths;
    /// let paths = VxPaths::with_base_dir("/tmp/vx");
    /// let ts_dir = paths.global_package_dir("npm", "typescript", "5.3.3");
    /// assert!(ts_dir.ends_with("packages/npm/typescript/5.3.3"));
    /// ```
    pub fn global_package_dir(&self, ecosystem: &str, package: &str, version: &str) -> PathBuf {
        self.ecosystem_packages_dir(ecosystem)
            .join(normalize_package_name(package))
            .join(version)
    }

    /// Get the bin directory for a global package
    ///
    /// Returns: ~/.vx/packages/{ecosystem}/{package}/{version}/bin
    pub fn global_package_bin_dir(&self, ecosystem: &str, package: &str, version: &str) -> PathBuf {
        self.global_package_dir(ecosystem, package, version)
            .join("bin")
    }

    /// Get the venv directory for a pip global package
    ///
    /// Returns: ~/.vx/packages/pip/{package}/{version}/venv
    pub fn global_pip_venv_dir(&self, package: &str, version: &str) -> PathBuf {
        self.global_package_dir("pip", package, version)
            .join("venv")
    }

    /// Get the node_modules directory for an npm global package
    ///
    /// Returns: ~/.vx/packages/npm/{package}/{version}/node_modules
    pub fn global_npm_node_modules_dir(&self, package: &str, version: &str) -> PathBuf {
        self.global_package_dir("npm", package, version)
            .join("node_modules")
    }

    /// Get the project-local bin directory
    ///
    /// Returns: {project_root}/.vx/bin
    pub fn project_bin_dir(&self, project_root: &Path) -> PathBuf {
        project_root.join(".vx").join("bin")
    }

    /// Get the global tools configuration file path
    ///
    /// Returns: ~/.vx/config/global-tools.toml
    pub fn global_tools_config(&self) -> PathBuf {
        self.config_dir.join("global-tools.toml")
    }

    /// Get the global packages registry file path
    ///
    /// Returns: ~/.vx/config/packages-registry.json
    pub fn packages_registry_file(&self) -> PathBuf {
        self.config_dir.join("packages-registry.json")
    }
}

impl Default for VxPaths {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback to current directory if home directory is not available
            Self::with_base_dir(".vx")
        })
    }
}

/// Get the executable file extension for the current platform
///
/// Deprecated: Use `platform::executable_extension()` instead.
#[deprecated(since = "0.6.0", note = "Use platform::executable_extension() instead")]
pub fn executable_extension_legacy() -> &'static str {
    platform::executable_extension()
}

/// Add executable extension to a tool name if needed
///
/// Deprecated: Use `platform::with_executable_extension()` instead.
#[deprecated(
    since = "0.6.0",
    note = "Use platform::with_executable_extension() instead"
)]
pub fn with_executable_extension_legacy(tool_name: &str) -> String {
    platform::with_executable_extension(tool_name)
}

/// Normalize package name for filesystem lookup
///
/// On Windows and macOS (case-insensitive filesystems), convert to lowercase.
/// On Linux, keep the original case.
pub fn normalize_package_name(name: &str) -> String {
    platform::normalize_for_comparison(name)
}
