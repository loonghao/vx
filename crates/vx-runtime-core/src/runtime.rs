//! Runtime trait definition
//!
//! The `Runtime` trait is the core abstraction for executable runtimes in vx.

use crate::ecosystem::Ecosystem;
use crate::platform::Platform;
use crate::traits::{CommandExecutor, FileSystem, HttpClient, Installer, PathProvider};
use crate::types::{ExecutionPrep, ExecutionResult, InstallResult, RuntimeDependency, VersionInfo};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Installation verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the installation is valid
    pub valid: bool,
    /// Path to the executable if found
    pub executable_path: Option<std::path::PathBuf>,
    /// List of issues found during verification
    pub issues: Vec<String>,
    /// Suggested fixes for the issues
    pub suggestions: Vec<String>,
}

impl VerificationResult {
    /// Create a successful verification result
    pub fn success(executable_path: std::path::PathBuf) -> Self {
        Self {
            valid: true,
            executable_path: Some(executable_path),
            issues: vec![],
            suggestions: vec![],
        }
    }

    /// Create a failed verification result
    pub fn failure(issues: Vec<String>, suggestions: Vec<String>) -> Self {
        Self {
            valid: false,
            executable_path: None,
            issues,
            suggestions,
        }
    }

    /// Create a successful verification result for system-installed tools
    pub fn success_system_installed() -> Self {
        Self {
            valid: true,
            executable_path: None,
            issues: vec![],
            suggestions: vec![],
        }
    }
}

/// Core trait for implementing runtime support
///
/// A Runtime represents an executable tool that can be installed and executed.
/// Only `name()` and `fetch_versions()` are required - all other methods have defaults.
#[async_trait]
pub trait Runtime: Send + Sync {
    // ========== Required Methods ==========

    /// Runtime name (e.g., "node", "go", "uv")
    fn name(&self) -> &str;

    /// Fetch available versions from the official source
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>>;

    // ========== Optional Methods with Defaults ==========

    /// Runtime description
    fn description(&self) -> &str {
        "A runtime"
    }

    /// Aliases for this runtime (e.g., "nodejs" for "node")
    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// Ecosystem this runtime belongs to
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Unknown
    }

    /// Dependencies on other runtimes
    fn dependencies(&self) -> &[RuntimeDependency] {
        &[]
    }

    /// Additional metadata
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Mirror configurations for alternative download sources
    fn mirror_urls(&self) -> Vec<vx_manifest::MirrorConfig> {
        vec![]
    }

    /// Get possible bin directory names for this runtime
    fn possible_bin_dirs(&self) -> Vec<&str> {
        vec!["bin"]
    }

    /// Get the store directory name for this runtime
    fn store_name(&self) -> &str {
        self.name()
    }

    /// Returns the platforms this runtime supports
    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::all_common()
    }

    /// Check if this runtime supports the given platform
    fn is_platform_supported(&self, platform: &Platform) -> bool {
        self.supported_platforms()
            .iter()
            .any(|p| p.matches(platform))
    }

    /// Check platform support and return an error if not supported
    fn check_platform_support(&self) -> Result<(), String> {
        let current = Platform::current();
        if self.is_platform_supported(&current) {
            Ok(())
        } else {
            let supported: Vec<String> = self
                .supported_platforms()
                .iter()
                .map(|p| format!("{}-{}", p.os.as_str(), p.arch.as_str()))
                .collect();
            Err(format!(
                "Runtime '{}' does not support the current platform ({}-{}). Supported platforms: {}",
                self.name(),
                current.os.as_str(),
                current.arch.as_str(),
                supported.join(", ")
            ))
        }
    }

    // ========== Executable Path Configuration ==========

    /// Get the base name of the executable (without extension)
    fn executable_name(&self) -> &str {
        self.name()
    }

    /// Get the list of executable extensions to search for on Windows
    fn executable_extensions(&self) -> &[&str] {
        &[".exe"]
    }

    /// Get the directory path (relative to install root) where the executable is located
    fn executable_dir_path(&self, _version: &str, _platform: &Platform) -> Option<String> {
        None
    }

    /// Get the relative path to the executable within the install directory
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        let exe_name = self.executable_name();
        let full_name = platform.executable_with_extensions(exe_name, self.executable_extensions());

        match self.executable_dir_path(version, platform) {
            Some(dir) => format!("{}/{}", dir, full_name),
            None => full_name,
        }
    }

    /// Get executable layout configuration from provider.toml (RFC 0019)
    ///
    /// Returns layout configuration for handling various executable file layouts.
    /// The actual resolution is done in vx-runtime.
    fn executable_layout(&self) -> Option<vx_manifest::LayoutConfig> {
        None
    }

    /// Return the post-install normalization configuration (RFC 0022)
    fn normalize_config(&self) -> Option<&vx_manifest::NormalizeConfig> {
        None
    }

    // ========== Lifecycle Hooks ==========

    /// Called before installation begins
    async fn pre_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let _ = (version, ctx);
        Ok(())
    }

    /// Called immediately after download and extraction, before verification
    fn post_extract(&self, _version: &str, _install_path: &std::path::PathBuf) -> Result<()> {
        Ok(())
    }

    /// Called after successful installation
    async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let _ = (version, ctx);
        Ok(())
    }

    /// Verify that an installation is valid and complete
    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_relative = self.executable_relative_path(version, platform);

        let path = install_path.join(&exe_relative);
        if path.exists() {
            VerificationResult::success(path)
        } else {
            VerificationResult::failure(
                vec![format!("Executable not found at {}", path.display())],
                vec![],
            )
        }
    }

    /// Called before uninstallation begins
    async fn pre_uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let _ = (version, ctx);
        Ok(())
    }

    /// Called after successful uninstallation
    async fn post_uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let _ = (version, ctx);
        Ok(())
    }

    /// Called before command execution
    async fn pre_execute(&self, args: &[String], ctx: &ExecutionContext) -> Result<()> {
        let _ = (args, ctx);
        Ok(())
    }

    /// Called after command execution
    async fn post_execute(
        &self,
        args: &[String],
        result: &ExecutionResult,
        ctx: &ExecutionContext,
    ) -> Result<()> {
        let _ = (args, result, ctx);
        Ok(())
    }

    /// Called before switching to a different version
    async fn pre_switch(
        &self,
        from_version: Option<&str>,
        to_version: &str,
        ctx: &RuntimeContext,
    ) -> Result<()> {
        let _ = (from_version, to_version, ctx);
        Ok(())
    }

    /// Called after switching to a different version
    async fn post_switch(
        &self,
        from_version: Option<&str>,
        to_version: &str,
        ctx: &RuntimeContext,
    ) -> Result<()> {
        let _ = (from_version, to_version, ctx);
        Ok(())
    }

    /// Called before updating to a new version
    async fn pre_update(
        &self,
        from_version: &str,
        to_version: &str,
        ctx: &RuntimeContext,
    ) -> Result<()> {
        let _ = (from_version, to_version, ctx);
        Ok(())
    }

    /// Called after successful update
    async fn post_update(
        &self,
        from_version: &str,
        to_version: &str,
        ctx: &RuntimeContext,
    ) -> Result<()> {
        let _ = (from_version, to_version, ctx);
        Ok(())
    }

    // ========== Core Operations ==========
    // Note: Default implementations for install, is_installed, execute, etc.
    // are provided in vx-runtime, not here, to keep this crate lightweight.

    /// Install a specific version
    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult>;

    /// Check if a version is installed
    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool>;

    /// Get installed versions
    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>>;

    /// Get the executable path for an installed version
    async fn get_executable_path_for_version(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<std::path::PathBuf>>;

    /// Execute the runtime with given arguments
    async fn execute(&self, args: &[String], ctx: &ExecutionContext) -> Result<ExecutionResult>;

    /// Pre-run hook called before executing a command
    async fn pre_run(&self, _args: &[String], _executable: &Path) -> Result<bool> {
        Ok(true)
    }

    /// Prepare environment variables for command execution
    async fn prepare_environment(
        &self,
        _version: &str,
        _ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    /// Prepare execution-specific environment variables
    async fn execution_environment(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        self.prepare_environment(version, ctx).await
    }

    /// Resolve a version specification to an actual installed version
    async fn resolve_installed_version(
        &self,
        version_spec: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<String>> {
        if version_spec == "latest" {
            let versions = self.installed_versions(ctx).await?;
            return Ok(versions.into_iter().max());
        }

        let versions = self.installed_versions(ctx).await?;
        if versions.contains(&version_spec.to_string()) {
            return Ok(Some(version_spec.to_string()));
        }

        for version in versions {
            if version.starts_with(version_spec) {
                return Ok(Some(version));
            }
        }

        Ok(None)
    }

    // ========== RFC 0028: Proxy-Managed and Bundled Runtimes ==========

    /// Check if a version can be directly installed by vx
    fn is_version_installable(&self, _version: &str) -> bool {
        true
    }

    /// Prepare execution for proxy-managed or bundled tool versions
    async fn prepare_execution(
        &self,
        _version: &str,
        _ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        Ok(ExecutionPrep::default())
    }

    /// Get download URL for a specific version and platform
    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        let _ = (version, platform);
        Ok(None)
    }

    /// Construct a download URL using a mirror's base URL
    async fn download_url_for_mirror(
        &self,
        _mirror_base_url: &str,
        _version: &str,
        _platform: &Platform,
    ) -> Result<Option<String>> {
        Ok(None)
    }

    /// Uninstall a specific version
    async fn uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()>;

    /// Resolve version string to actual version
    async fn resolve_version(&self, version: &str, ctx: &RuntimeContext) -> Result<String>;
}

/// Runtime context for dependency injection
pub struct RuntimeContext {
    /// Path provider
    pub paths: Arc<dyn PathProvider>,
    /// HTTP client
    pub http: Arc<dyn HttpClient>,
    /// File system operations
    pub fs: Arc<dyn FileSystem>,
    /// Installer
    pub installer: Arc<dyn Installer>,
    /// Cached download URL from lock file
    pub cached_download_url: Option<String>,
}

impl RuntimeContext {
    /// Get cached download URL for a tool
    pub fn get_cached_download_url(&self, _tool_name: &str) -> Option<String> {
        self.cached_download_url.clone()
    }
}

/// Execution context for command execution
pub struct ExecutionContext {
    /// Working directory
    pub working_dir: Option<std::path::PathBuf>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Whether to capture output
    pub capture_output: bool,
    /// Command executor
    pub executor: Arc<dyn CommandExecutor>,
}
