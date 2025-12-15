//! Runtime trait definition
//!
//! The `Runtime` trait is the core abstraction for executable runtimes in vx.

use crate::context::{ExecutionContext, RuntimeContext};
use crate::ecosystem::Ecosystem;
use crate::platform::Platform;
use crate::types::{ExecutionResult, InstallResult, RuntimeDependency, VersionInfo};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Core trait for implementing runtime support
///
/// A Runtime represents an executable tool that can be installed and executed,
/// such as Node.js, Go, or UV.
///
/// # Required Methods
///
/// Only two methods are required:
/// - `name()`: Return the runtime name
/// - `fetch_versions()`: Fetch available versions from the official source
///
/// All other methods have sensible defaults.
///
/// # Example
///
/// ```rust,ignore
/// use vx_runtime::{Runtime, RuntimeContext, VersionInfo, Ecosystem};
/// use async_trait::async_trait;
///
/// struct NodeRuntime;
///
/// #[async_trait]
/// impl Runtime for NodeRuntime {
///     fn name(&self) -> &str {
///         "node"
///     }
///
///     fn ecosystem(&self) -> Ecosystem {
///         Ecosystem::NodeJs
///     }
///
///     fn aliases(&self) -> &[&str] {
///         &["nodejs"]
///     }
///
///     async fn fetch_versions(&self, ctx: &RuntimeContext) -> anyhow::Result<Vec<VersionInfo>> {
///         // Fetch from nodejs.org API
///         let response = ctx.http.get_json_value("https://nodejs.org/dist/index.json").await?;
///         // Parse and return versions
///         Ok(vec![])
///     }
/// }
/// ```
#[async_trait]
pub trait Runtime: Send + Sync {
    // ========== Required Methods ==========

    /// Runtime name (e.g., "node", "go", "uv")
    ///
    /// This should be the primary name used to invoke the runtime.
    fn name(&self) -> &str;

    /// Fetch available versions from the official source
    ///
    /// This method should fetch version information from the runtime's
    /// official API or release page.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Runtime context providing HTTP client and other dependencies
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
    ///
    /// For example, npm depends on node.
    fn dependencies(&self) -> &[RuntimeDependency] {
        &[]
    }

    /// Additional metadata
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Get the relative path to the executable within the install directory
    ///
    /// Override this if your runtime's archive extracts to a non-standard layout.
    /// Default is `bin/{name}` (or `bin/{name}.exe` on Windows).
    ///
    /// # Examples
    ///
    /// - Node.js: `bin/node` (default)
    /// - UV: `uv-{platform}/uv` (custom)
    /// - Bun: `bun-{platform}/bun` (custom)
    fn executable_relative_path(&self, _version: &str, _platform: &Platform) -> String {
        let exe_name = if cfg!(windows) {
            format!("{}.exe", self.name())
        } else {
            self.name().to_string()
        };
        format!("bin/{}", exe_name)
    }

    // ========== Lifecycle Hooks ==========
    //
    // All hooks have default empty implementations.
    // Providers can override these to add custom behavior.
    // Return `Err` from pre_* hooks to abort the operation.

    // --- Install Hooks ---

    /// Called before installation begins
    ///
    /// Use this to:
    /// - Check system dependencies
    /// - Validate environment
    /// - Clean up previous failed installations
    /// - Create necessary directories
    ///
    /// Return `Err` to abort installation.
    async fn pre_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let _ = (version, ctx);
        Ok(())
    }

    /// Called after successful installation
    ///
    /// Use this to:
    /// - Set up PATH or symlinks
    /// - Run initialization scripts
    /// - Verify the installation works
    /// - Install bundled tools (e.g., npm with node)
    async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let _ = (version, ctx);
        Ok(())
    }

    // --- Uninstall Hooks ---

    /// Called before uninstallation begins
    ///
    /// Use this to:
    /// - Check if other tools depend on this version
    /// - Backup configuration files
    /// - Warn about data loss
    ///
    /// Return `Err` to abort uninstallation.
    async fn pre_uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let _ = (version, ctx);
        Ok(())
    }

    /// Called after successful uninstallation
    ///
    /// Use this to:
    /// - Remove symlinks
    /// - Clean up cache files
    /// - Update global configuration
    async fn post_uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let _ = (version, ctx);
        Ok(())
    }

    // --- Execute Hooks ---

    /// Called before command execution
    ///
    /// Use this to:
    /// - Set up environment variables
    /// - Validate arguments
    /// - Check prerequisites
    /// - Log execution start
    ///
    /// Return `Err` to abort execution.
    async fn pre_execute(&self, args: &[String], ctx: &ExecutionContext) -> Result<()> {
        let _ = (args, ctx);
        Ok(())
    }

    /// Called after command execution (regardless of success/failure)
    ///
    /// Use this to:
    /// - Clean up temporary files
    /// - Log execution results
    /// - Update statistics
    async fn post_execute(
        &self,
        args: &[String],
        result: &ExecutionResult,
        ctx: &ExecutionContext,
    ) -> Result<()> {
        let _ = (args, result, ctx);
        Ok(())
    }

    // --- Switch/Use Hooks ---

    /// Called before switching to a different version
    ///
    /// Use this to:
    /// - Validate the target version exists
    /// - Check compatibility
    /// - Backup current configuration
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
    ///
    /// Use this to:
    /// - Update symlinks
    /// - Rehash shell commands
    /// - Notify user of changes
    async fn post_switch(
        &self,
        from_version: Option<&str>,
        to_version: &str,
        ctx: &RuntimeContext,
    ) -> Result<()> {
        let _ = (from_version, to_version, ctx);
        Ok(())
    }

    // --- Update Hooks ---

    /// Called before updating to a new version
    ///
    /// Use this to:
    /// - Check if update is available
    /// - Backup current version
    /// - Validate update path
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
    ///
    /// Use this to:
    /// - Migrate configuration
    /// - Clean up old version (optional)
    /// - Verify new version works
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

    /// Install a specific version
    ///
    /// Default implementation downloads and extracts to the store.
    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        use tracing::{debug, info};

        let install_path = ctx.paths.version_store_dir(self.name(), version);
        let platform = Platform::current();
        let exe_relative = self.executable_relative_path(version, &platform);
        let exe_path = install_path.join(&exe_relative);

        debug!(
            "Install path for {} {}: {}",
            self.name(),
            version,
            install_path.display()
        );
        debug!("Executable relative path: {}", exe_relative);

        // Check if already installed
        if ctx.fs.exists(&install_path) {
            // Verify the executable actually exists
            if ctx.fs.exists(&exe_path) {
                debug!("Already installed: {}", exe_path.display());
                return Ok(InstallResult::already_installed(
                    install_path,
                    exe_path,
                    version.to_string(),
                ));
            } else {
                // Directory exists but executable doesn't - clean up and reinstall
                debug!(
                    "Install directory exists but executable missing, cleaning up: {}",
                    install_path.display()
                );
                if let Err(e) = std::fs::remove_dir_all(&install_path) {
                    debug!("Failed to clean up directory: {}", e);
                }
            }
        }

        // Get download URL
        debug!("Platform: {:?}", platform);

        let url = self
            .download_url(version, &platform)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No download URL for {} {}", self.name(), version))?;

        info!("Downloading {} {} from {}", self.name(), version, url);

        // Download and extract
        ctx.installer
            .download_and_extract(&url, &install_path)
            .await?;

        debug!("Expected executable path: {}", exe_path.display());

        Ok(InstallResult::success(
            install_path,
            exe_path,
            version.to_string(),
        ))
    }

    /// Check if a version is installed
    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        let install_path = ctx.paths.version_store_dir(self.name(), version);
        Ok(ctx.fs.exists(&install_path))
    }

    /// Get installed versions
    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>> {
        let runtime_dir = ctx.paths.runtime_store_dir(self.name());
        if !ctx.fs.exists(&runtime_dir) {
            return Ok(vec![]);
        }

        let entries = ctx.fs.read_dir(&runtime_dir)?;
        let mut versions: Vec<String> = entries
            .into_iter()
            .filter(|p| ctx.fs.is_dir(p))
            .filter_map(|p| p.file_name().and_then(|n| n.to_str().map(String::from)))
            .collect();

        // Sort versions (newest first)
        versions.sort_by(|a, b| b.cmp(a));
        Ok(versions)
    }

    /// Execute the runtime with given arguments
    async fn execute(&self, args: &[String], ctx: &ExecutionContext) -> Result<ExecutionResult> {
        ctx.executor
            .execute(
                self.name(),
                args,
                ctx.working_dir.as_deref(),
                &ctx.env,
                ctx.capture_output,
            )
            .await
    }

    /// Get download URL for a specific version and platform
    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Default implementation - subclasses should override
        let _ = (version, platform);
        Ok(None)
    }

    /// Uninstall a specific version
    async fn uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let install_path = ctx.paths.version_store_dir(self.name(), version);
        if ctx.fs.exists(&install_path) {
            ctx.fs.remove_dir_all(&install_path)?;
        }
        Ok(())
    }

    /// Resolve "latest" to actual version
    async fn resolve_version(&self, version: &str, ctx: &RuntimeContext) -> Result<String> {
        if version == "latest" {
            let versions = self.fetch_versions(ctx).await?;
            versions
                .into_iter()
                .filter(|v| !v.prerelease)
                .map(|v| v.version)
                .next()
                .ok_or_else(|| anyhow::anyhow!("No versions found for {}", self.name()))
        } else {
            Ok(version.to_string())
        }
    }
}
