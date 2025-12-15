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

    /// Install a specific version
    ///
    /// Default implementation downloads and extracts to the store.
    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let install_path = ctx.paths.version_store_dir(self.name(), version);

        // Check if already installed
        if ctx.fs.exists(&install_path) {
            let exe_path = ctx.paths.executable_path(self.name(), version);
            return Ok(InstallResult::already_installed(
                install_path,
                exe_path,
                version.to_string(),
            ));
        }

        // Get download URL
        let platform = Platform::current();
        let url = self
            .download_url(version, &platform)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No download URL for {} {}", self.name(), version))?;

        // Download and extract
        ctx.installer
            .download_and_extract(&url, &install_path)
            .await?;

        let exe_path = ctx.paths.executable_path(self.name(), version);
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
