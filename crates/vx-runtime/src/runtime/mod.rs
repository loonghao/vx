//! Runtime trait definition
//!
//! The `Runtime` trait is the core abstraction for executable runtimes in vx.
//!
//! # ISP Sub-traits
//!
//! [`Runtime`] is large by necessity, but callers that only need a subset of its
//! capabilities can use the narrower sub-traits defined in [`subtrait`]:
//!
//! - [`subtrait::RuntimeIdentity`] — name, description, aliases, ecosystem, …
//! - [`subtrait::RuntimePlatform`] — platform support checks
//! - [`subtrait::RuntimeVersioning`] — fetch, resolve and list versions
//! - [`subtrait::RuntimeExecutable`] — executable path configuration
//! - [`subtrait::RuntimeInstallable`] — download, install, uninstall
//! - [`subtrait::RuntimeHooks`] — lifecycle hooks (pre/post install, execute, …)
//! - [`subtrait::RuntimeEnvironment`] — environment variable preparation
//! - [`subtrait::RuntimeExecuteOps`] — command execution
//! - [`subtrait::RuntimeShell`] — shell provider (RFC 0038)
//!
//! Every type that implements `Runtime` satisfies all sub-traits automatically via
//! blanket implementations — no changes needed to existing implementors.
//!
//! # Executable Path Resolution
//!
//! The framework provides a unified approach to handle executable paths across platforms:
//!
//! 1. **Simple case**: Override `executable_name()` to return the base name (e.g., "node")
//! 2. **Custom extensions**: Override `executable_extensions()` for tools that use `.cmd`/`.bat` on Windows
//! 3. **Custom directory**: Override `executable_dir_path()` if the executable is not in the root
//! 4. **Full control**: Override `executable_relative_path()` for complex cases
//!
//! The framework automatically handles:
//! - Platform-specific extensions (`.exe`, `.cmd`, `.bat` on Windows)
//! - Searching for executables in install directories
//! - Verification of installations

pub mod install_impl;
pub mod subtrait;
pub mod verify;

pub use subtrait::{
    RuntimeEnvironment, RuntimeExecutable, RuntimeExecuteOps, RuntimeHooks, RuntimeIdentity,
    RuntimeInstallable, RuntimePlatform, RuntimeShell, RuntimeVersioning,
};
pub use verify::VerificationResult;

use crate::context::{ExecutionContext, RuntimeContext};
use crate::ecosystem::Ecosystem;
use crate::platform::Platform;
use crate::region;
use crate::types::{
    ExecutionPrep, ExecutionResult, InstallResult, RuntimeDependency, VersionInfo,
    VersionInfoResult,
};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime_core::{MirrorConfig, NormalizeConfig};
use vx_versions::VersionResolver;

/// Detect the download region for mirror selection
///
/// Returns the region string used in provider.toml mirror configs (e.g., "cn", "global")
fn detect_download_region() -> &'static str {
    match region::detect_region() {
        region::Region::China => "cn",
        region::Region::Global => "global",
    }
}

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
///     fn aliases(&self) -> Vec<&str> {
///         vec!["nodejs"]
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

    /// Aliases for this runtime (e.g., "nodejs" for "node").
    ///
    /// Returns a `Vec<&str>` borrowed from self, which works for both
    /// compile-time static aliases and runtime-dynamic aliases (e.g., loaded
    /// from a manifest file). Override with `vec!["alias1", "alias2"]` for
    /// static aliases, or `self.aliases.iter().map(|s| s.as_str()).collect()`
    /// for dynamically stored aliases.
    fn aliases(&self) -> Vec<&str> {
        vec![]
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

    /// Version-aware dependencies resolved at runtime.
    ///
    /// This is used for providers whose dependency rules depend on the selected
    /// version in `provider.star::deps(ctx, version)`.
    async fn versioned_dependencies(
        &self,
        _version: &str,
        _ctx: &RuntimeContext,
    ) -> Result<Vec<RuntimeDependency>> {
        Ok(vec![])
    }

    /// Resolve version indirection for toolchain-managed tools (RFC 0040).
    ///
    /// For most tools, returns `None` (1:1 mapping: store_version = user_version).
    ///
    /// For tools where the user specifies the managed tool's version but vx
    /// must download a manager/installer (e.g., Rust: user writes rustc version,
    /// vx downloads rustup installer), this method returns a `VersionInfoResult`
    /// describing:
    /// - `store_as`: the directory name under `~/.vx/store/<tool>/`
    /// - `download_version`: which installer version to download (None = latest)
    /// - `install_params`: extra params passed to `post_extract`
    ///
    /// This enables O(1) version detection in `vx check` and eliminates the
    /// need for special-case passthrough logic in `vx lock`.
    async fn version_info(
        &self,
        _user_version: &str,
        _ctx: &RuntimeContext,
    ) -> Result<Option<VersionInfoResult>> {
        Ok(None)
    }

    /// Additional metadata
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Mirror configurations for alternative download sources
    ///
    /// Returns a list of mirror URLs configured in the provider manifest.
    /// Each mirror has a region (e.g., "cn" for China) and a base URL.
    ///
    /// When installing, the framework will automatically select the best mirror
    /// based on the user's detected region, falling back to the original URL
    /// if the mirror fails.
    ///
    /// Override this method to provide mirrors for your runtime.
    fn mirror_urls(&self) -> Vec<MirrorConfig> {
        vec![]
    }

    /// Get possible bin directory names for this runtime
    ///
    /// Returns a list of possible subdirectory names where executables might be located.
    /// The first matching directory will be used.
    ///
    /// Default implementation returns ["bin"], but providers can override this
    /// for runtimes with non-standard directory structures.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn possible_bin_dirs(&self) -> Vec<&str> {
    ///     match self.name() {
    ///         "python" => vec!["python", "bin"],
    ///         "uv" => vec!["bin"],
    ///         _ => vec!["bin"],
    ///     }
    /// }
    /// ```
    fn possible_bin_dirs(&self) -> Vec<&str> {
        vec!["bin"]
    }

    /// Get the store directory name for this runtime
    ///
    /// This is the canonical name used for the store directory path.
    /// For bundled runtimes (e.g., npm, npx bundled with node), this returns
    /// the parent runtime's name. For standalone runtimes, this returns `self.name()`.
    ///
    /// **IMPORTANT**: Always use this method (not `name()`) when constructing store paths
    /// to ensure consistency between installation and lookup.
    ///
    /// # Examples
    ///
    /// - `node.store_name()` → `"node"` (standalone)
    /// - `npm.store_name()` → `"node"` (bundled with node)
    /// - `uvx.store_name()` → `"uv"` (bundled with uv)
    /// - `vscode.store_name()` → `"code"` (alias, canonical name is "code")
    fn store_name(&self) -> &str {
        // Check if bundled_with is set in metadata
        // Note: This has a limitation - can't return &str from owned String
        // Providers that use bundled_with should override this method directly
        self.name()
    }

    /// Resolve a version specification to an actual installed version
    ///
    /// This method handles version resolution including:
    /// - "latest": resolve to the latest installed version
    /// - Partial versions: "3.11" -> "3.11.14"
    /// - Exact versions: return as-is if installed
    ///
    /// Returns the actual version string that should be used for path construction.
    /// Returns None if no matching version is installed.
    ///
    /// # Arguments
    ///
    /// * `version_spec` - Version specification from user (e.g., "3.11", "latest")
    /// * `ctx` - Runtime context for accessing installed versions
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Assuming 3.11.14 is installed
    /// assert_eq!(runtime.resolve_installed_version("3.11", ctx).await?, Some("3.11.14".to_string()));
    /// assert_eq!(runtime.resolve_installed_version("latest", ctx).await?, Some("3.12.0".to_string()));
    /// ```
    async fn resolve_installed_version(
        &self,
        version_spec: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<String>> {
        // Default implementation: try exact match, then prefix match
        if version_spec == "latest" {
            let versions = self.installed_versions(ctx).await?;
            return Ok(versions.into_iter().max());
        }

        // Try exact match first
        let versions = self.installed_versions(ctx).await?;
        if versions.contains(&version_spec.to_string()) {
            return Ok(Some(version_spec.to_string()));
        }

        // Try prefix match (e.g., "3.11" matches "3.11.14")
        for version in versions {
            if version.starts_with(version_spec) {
                return Ok(Some(version));
            }
        }

        Ok(None)
    }

    /// Returns the platforms this runtime supports
    ///
    /// By default, returns all common platforms (Windows, macOS, Linux on x64 and arm64).
    /// Override this for platform-specific tools (e.g., Chocolatey is Windows-only).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fn supported_platforms(&self) -> Vec<Platform> {
    ///     // Windows-only tool
    ///     Platform::windows_only()
    /// }
    /// ```
    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::all_common()
    }

    /// Check if this runtime supports the given platform
    ///
    /// Default implementation checks if the platform is in `supported_platforms()`.
    fn is_platform_supported(&self, platform: &Platform) -> bool {
        self.supported_platforms()
            .iter()
            .any(|p| p.matches(platform))
    }

    /// Check platform support and return an error if not supported
    ///
    /// This is a convenience method that returns a detailed error message
    /// when the current platform is not supported.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // In your provider code:
    /// runtime.check_platform_support()?;
    /// ```
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
    //
    // These methods provide a layered approach to configure executable paths.
    // Most providers only need to override one or two of these methods.

    /// Get the base name of the executable (without extension)
    ///
    /// Default returns `self.name()`. Override if the executable name differs
    /// from the runtime name.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn executable_name(&self) -> &str {
    ///     "python3"  // Different from runtime name "python"
    /// }
    /// ```
    fn executable_name(&self) -> &str {
        self.name()
    }

    /// Get the list of executable extensions to search for on Windows
    ///
    /// Default returns `[".exe"]`. Override for tools that use `.cmd` or `.bat`.
    /// On non-Windows platforms, this is ignored and no extension is used.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn executable_extensions(&self) -> &[&str] {
    ///     &[".cmd", ".exe"]  // npm, npx, yarn use .cmd on Windows
    /// }
    /// ```
    fn executable_extensions(&self) -> &[&str] {
        &[".exe"]
    }

    /// Get the directory path (relative to install root) where the executable is located
    ///
    /// Default returns `None`, meaning the executable is in the install root.
    /// Override to specify a subdirectory.
    ///
    /// # Arguments
    /// * `version` - The version being installed
    /// * `platform` - The target platform
    ///
    /// # Example
    /// ```rust,ignore
    /// fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String> {
    ///     // Node.js: node-v22.0.0-linux-x64/bin/
    ///     let dir = format!("node-v{}-{}", version, platform.as_str());
    ///     if platform.is_windows() {
    ///         Some(dir)  // Windows: no bin subdirectory
    ///     } else {
    ///         Some(format!("{}/bin", dir))
    ///     }
    /// }
    /// ```
    fn executable_dir_path(&self, _version: &str, _platform: &Platform) -> Option<String> {
        None
    }

    /// Get the relative path to the executable within the install directory
    ///
    /// **Most providers should NOT override this method.**
    /// Instead, override `executable_name()`, `executable_extensions()`, `executable_dir_path()`,
    /// or `executable_layout()` (RFC 0019).
    ///
    /// This method combines the above methods to construct the full relative path.
    /// Only override this for complex cases that can't be handled by the simpler methods.
    ///
    /// # Default behavior
    /// 1. If `executable_layout()` is provided, use it to resolve the path
    /// 2. Otherwise, construct path from `executable_dir_path()` + `executable_name()` + extension
    /// - On Windows: tries extensions from `executable_extensions()` in order
    /// - On Unix: no extension
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        // Try layout-based resolution first (RFC 0019)
        if let Some(layout) = self.executable_layout() {
            use crate::layout::LayoutContext;

            let ctx = LayoutContext {
                version: version.to_string(),
                name: self.name().to_string(),
                platform: platform.clone(),
            };

            if let Ok(resolved) = layout.resolve(&ctx) {
                // Return the relative path from resolved layout
                let install_root = std::path::Path::new("");
                let exe_path = resolved.executable_path(install_root);
                return exe_path.to_string_lossy().to_string();
            }
        }

        // Fallback to legacy method
        let exe_name = self.executable_name();
        let full_name = platform.executable_with_extensions(exe_name, self.executable_extensions());

        match self.executable_dir_path(version, platform) {
            Some(dir) => format!("{}/{}", dir, full_name),
            None => full_name,
        }
    }

    // ========== RFC 0019: Executable Layout Configuration ==========

    /// Get executable layout configuration from provider.toml (RFC 0019)
    ///
    /// This provides a declarative way to configure executable file layouts
    /// instead of hardcoding in Rust. Supports:
    /// - Single binary files with renaming (e.g., yasm-1.3.0-win64.exe → bin/yasm.exe)
    /// - Archives with nested directories (e.g., ffmpeg-6.0/bin/ffmpeg.exe)
    /// - Platform-specific layouts
    /// - Variable interpolation ({version}, {name}, {platform}, etc.)
    ///
    /// # Example (provider.toml)
    /// ```toml
    /// [runtimes.layout]
    /// download_type = "binary"
    ///
    /// [runtimes.layout.binary."windows-x86_64"]
    /// source_name = "tool-{version}-win64.exe"
    /// target_name = "tool.exe"
    /// target_dir = "bin"
    /// ```
    ///
    /// Most providers should return `None` and rely on the default path resolution.
    /// Only override this for tools with complex file layouts.
    fn executable_layout(&self) -> Option<crate::layout::ExecutableLayout> {
        None
    }

    /// Return the post-install normalization configuration (RFC 0022)
    ///
    /// Normalization ensures a consistent directory structure after installation:
    /// - Creates a standard `bin/` directory
    /// - Links or copies executables to standard locations
    /// - Creates aliases for additional commands
    ///
    /// # Example provider.toml
    ///
    /// ```toml
    /// [runtimes.normalize]
    /// enabled = true
    ///
    /// [[runtimes.normalize.executables]]
    /// source = "ImageMagick-*-Q16-HDRI/magick.exe"
    /// target = "magick.exe"
    /// action = "link"
    ///
    /// [[runtimes.normalize.aliases]]
    /// name = "convert"
    /// target = "magick"
    /// ```
    fn normalize_config(&self) -> Option<&NormalizeConfig> {
        None
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

    /// Called immediately after download and extraction, before verification
    ///
    /// This is a synchronous hook that runs before the installation is verified.
    /// Use this for operations that must happen before verification, such as:
    /// - Renaming downloaded files to standard names (e.g., pnpm-macos-arm64 -> pnpm)
    /// - Setting executable permissions
    /// - Moving files to expected locations
    ///
    /// Unlike `post_install`, this runs before verification and is synchronous.
    fn post_extract(&self, _version: &str, _install_path: &std::path::Path) -> Result<()> {
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

    /// Verify that an installation is valid and complete.
    ///
    /// Checks that the executable exists at the expected path (supports glob patterns)
    /// and is executable on Unix. Override for custom verification logic.
    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_relative = self.executable_relative_path(version, platform);
        verify::verify_installation_default(
            &exe_relative,
            install_path,
            self.executable_name(),
            self.executable_extensions(),
        )
    }

    /// Helper to find executable in install directory (searches up to 3 levels deep).
    fn find_executable_in_install_dir(&self, install_path: &Path) -> Option<std::path::PathBuf> {
        verify::find_executable_in_install_dir(
            install_path,
            self.executable_name(),
            self.executable_extensions(),
        )
    }

    /// Recursively search for an executable.
    fn search_for_executable(
        &self,
        dir: &Path,
        exe_name: &str,
        current_depth: usize,
        max_depth: usize,
    ) -> Option<std::path::PathBuf> {
        verify::search_for_executable(dir, exe_name, self.name(), current_depth, max_depth)
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

    /// Install a specific version.
    ///
    /// Default implementation downloads and extracts to the store.
    /// See [`install_impl::default_install_inner`] for the full logic.
    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        use install_impl::{
            InstallParams, build_layout_metadata, default_install_inner,
            is_url_plausible_for_platform,
        };

        // Fail early with a clear message when the provider doesn't support this platform.
        if let Err(msg) = self.check_platform_support() {
            return Err(anyhow::anyhow!(msg));
        }

        let platform = Platform::current();
        let exe_relative = self.executable_relative_path(version, &platform);

        // Resolve download URL (cached lock file → runtime method)
        let url = if let Some(cached_url) = ctx
            .get_cached_download_url(self.name())
            .filter(|u| is_url_plausible_for_platform(u, &platform))
        {
            cached_url
        } else if let Some(cached_url) = ctx
            .get_cached_download_url(self.store_name())
            .filter(|u| is_url_plausible_for_platform(u, &platform))
        {
            cached_url
        } else {
            self.download_url(version, &platform)
                .await?
                .ok_or_else(|| anyhow::anyhow!("No download URL for {} {}", self.name(), version))?
        };

        let download_urls = self
            .build_download_url_chain(&url, version, &platform)
            .await;
        let layout = self.executable_layout();
        let layout_metadata =
            build_layout_metadata(layout.as_ref(), version, self.name(), &platform);

        let params = InstallParams {
            name: self.name(),
            store_name: self.store_name(),
            exe_relative,
            exe_name: self.executable_name(),
            exe_extensions: self.executable_extensions(),
            layout_metadata,
            download_urls,
            normalize_config: self.normalize_config(),
        };

        default_install_inner(params, version, ctx, |v, p| self.post_extract(v, p)).await
    }

    /// Check if a version is installed
    ///
    /// If version is "latest", checks if any version is installed.
    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        // Handle "latest" by checking if any version is installed
        if version == "latest" {
            let versions = self.installed_versions(ctx).await?;
            return Ok(!versions.is_empty());
        }

        // New layout: install directly to version dir; fallback to platform dir for old installs.
        let platform = Platform::current();
        let base_path = ctx.paths.version_store_dir(self.store_name(), version);
        let platform_dir = base_path.join(platform.as_str());
        let exists = ctx.fs.exists(&base_path) || ctx.fs.exists(&platform_dir);
        Ok(exists)
    }

    /// Get the executable path for an installed version
    ///
    /// Returns the absolute path to the executable for the specified version.
    /// This method should be overridden by runtimes that install to non-standard
    /// locations (e.g., Python installed via uv).
    ///
    /// The default implementation looks in the vx store directory.
    async fn get_executable_path_for_version(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<std::path::PathBuf>> {
        // New layout: install directly to version dir; fallback to platform dir for old installs.
        let platform = Platform::current();
        let base_path = ctx.paths.version_store_dir(self.store_name(), version);
        let platform_dir = base_path.join(platform.as_str());
        let install_path = if base_path.exists() {
            base_path
        } else {
            platform_dir
        };
        if !ctx.fs.exists(&install_path) {
            return Ok(None);
        }

        let platform = Platform::current();
        let verification = self.verify_installation(version, &install_path, &platform);

        if verification.valid {
            Ok(verification.executable_path)
        } else {
            Ok(None)
        }
    }

    /// Get installed versions
    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>> {
        let runtime_dir = ctx.paths.runtime_store_dir(self.store_name());
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
        #[allow(clippy::unnecessary_sort_by)]
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

    /// Pre-run hook called before executing a command
    ///
    /// This hook is called by the executor before running a command.
    /// Providers can use this to perform setup tasks like:
    /// - Syncing dependencies (e.g., `uv sync` before `uv run`)
    /// - Setting up virtual environments
    /// - Checking prerequisites
    ///
    /// # Arguments
    ///
    /// * `args` - The command arguments (e.g., ["run", "pytest"] for `uv run pytest`)
    /// * `executable` - Path to the resolved executable
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Continue with execution
    /// * `Ok(false)` - Skip execution (pre_run handled everything)
    /// * `Err(...)` - Abort with error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn pre_run(&self, args: &[String], executable: &Path) -> Result<bool> {
    ///     // For "uv run" commands, ensure dependencies are synced
    ///     if args.first().is_some_and(|a| a == "run") {
    ///         if Path::new("pyproject.toml").exists() && !Path::new(".venv").exists() {
    ///             // Run uv sync first
    ///             Command::new(executable).arg("sync").status().await?;
    ///         }
    ///     }
    ///     Ok(true) // Continue with execution
    /// }
    /// ```
    async fn pre_run(&self, _args: &[String], _executable: &Path) -> Result<bool> {
        Ok(true) // Default: no pre-run action, continue execution
    }

    /// Prepare environment variables for command execution
    ///
    /// This method is called by the executor before running a command to get
    /// any additional environment variables that the runtime requires.
    ///
    /// Most runtimes don't need this - they work with just the executable path.
    /// However, some runtimes like MSVC require specific environment variables
    /// (INCLUDE, LIB, PATH) to function properly.
    ///
    /// # Arguments
    ///
    /// * `version` - The version of the runtime being executed
    /// * `ctx` - Runtime context providing paths and other dependencies
    ///
    /// # Returns
    ///
    /// A HashMap of environment variable names to values that should be set
    /// before executing the runtime.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn prepare_environment(
    ///     &self,
    ///     version: &str,
    ///     ctx: &RuntimeContext,
    /// ) -> Result<HashMap<String, String>> {
    ///     let mut env = HashMap::new();
    ///
    ///     // Set INCLUDE path for MSVC
    ///     env.insert("INCLUDE".to_string(), "/path/to/includes".to_string());
    ///
    ///     // Set LIB path for MSVC
    ///     env.insert("LIB".to_string(), "/path/to/libs".to_string());
    ///
    ///     Ok(env)
    /// }
    /// ```
    async fn prepare_environment(
        &self,
        _version: &str,
        _ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        Ok(HashMap::new()) // Default: no extra environment variables
    }

    /// Prepare execution-specific environment variables for this runtime
    ///
    /// Unlike `prepare_environment()` which is used for general env setup
    /// (e.g., in `vx dev`), this method is called only when the runtime itself
    /// is being directly invoked as the primary command.
    ///
    /// This allows runtimes like MSVC to inject LIB/INCLUDE/PATH only when
    /// their tools (cl, link, nmake) are directly used, without polluting
    /// the environment of other tools like node-gyp.
    ///
    /// Default: delegates to `prepare_environment()`.
    ///
    /// See: <https://github.com/loonghao/vx/issues/573>
    async fn execution_environment(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        self.prepare_environment(version, ctx).await
    }

    // ========== RFC 0028: Proxy-Managed and Bundled Runtimes ==========

    /// Check if a version can be directly installed by vx
    ///
    /// Returns `true` (default): vx will download and install this version
    /// Returns `false`: vx will use a proxy mechanism instead (e.g., corepack for Yarn 2.x+)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fn is_version_installable(&self, version: &str) -> bool {
    ///     // Yarn 1.x is directly installable, 2.x+ uses corepack
    ///     version.starts_with('1')
    /// }
    /// ```
    fn is_version_installable(&self, _version: &str) -> bool {
        true // Default: all versions are directly installable
    }

    /// Prepare execution for proxy-managed or bundled tool versions
    ///
    /// This method is called before executing a version that returns `false` from
    /// `is_version_installable()`. It should:
    /// 1. Ensure the proxy mechanism is ready (e.g., enable corepack)
    /// 2. Return configuration for how to execute the tool
    ///
    /// # Arguments
    ///
    /// * `version` - The version being executed
    /// * `ctx` - Execution context with working directory and environment
    ///
    /// # Returns
    ///
    /// `ExecutionPrep` struct containing:
    /// - `use_system_path`: Whether to use system PATH instead of vx-managed path
    /// - `executable_override`: Optional direct path to the executable
    /// - `env_vars`: Additional environment variables
    /// - `command_prefix`: Command prefix (e.g., `["dotnet"]` for msbuild)
    /// - `proxy_ready`: Whether the proxy is ready for execution
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn prepare_execution(
    ///     &self,
    ///     version: &str,
    ///     ctx: &ExecutionContext,
    /// ) -> Result<ExecutionPrep> {
    ///     // For Yarn 2.x+, enable corepack and use system PATH
    ///     if !Self::is_corepack_enabled().await {
    ///         Self::enable_corepack().await?;
    ///     }
    ///     Ok(ExecutionPrep::proxy_ready())
    /// }
    /// ```
    async fn prepare_execution(
        &self,
        _version: &str,
        _ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        Ok(ExecutionPrep::default()) // Default: no special preparation needed
    }

    /// Get download URL for a specific version and platform
    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Default implementation - subclasses should override
        let _ = (version, platform);
        Ok(None)
    }

    /// Construct a download URL using a mirror's base URL
    ///
    /// Given a mirror base URL (e.g., `https://npmmirror.com/mirrors/node`),
    /// construct the full download URL for the specified version and platform.
    ///
    /// Default implementation returns `None` (mirror not supported).
    /// Providers should override this to support mirror downloads.
    ///
    /// # Example
    ///
    /// For Node.js with mirror base `https://npmmirror.com/mirrors/node`:
    /// ```text
    /// → https://npmmirror.com/mirrors/node/v22.0.0/node-v22.0.0-linux-x64.tar.gz
    /// ```
    async fn download_url_for_mirror(
        &self,
        _mirror_base_url: &str,
        _version: &str,
        _platform: &Platform,
    ) -> Result<Option<String>> {
        Ok(None)
    }

    /// Build a download URL chain with mirror fallback support
    ///
    /// Returns a list of URLs to try in order:
    /// 1. Region-matching mirror URLs (if in China and mirrors configured)
    /// 2. Original download URL (always last as fallback)
    ///
    /// This enables automatic mirror selection based on the user's region,
    /// with transparent fallback to the original source.
    async fn build_download_url_chain(
        &self,
        original_url: &str,
        version: &str,
        platform: &Platform,
    ) -> Vec<String> {
        let mirrors = self.mirror_urls();
        if mirrors.is_empty() {
            return vec![original_url.to_string()];
        }

        // Detect current region
        let detected_region = detect_download_region();

        // Filter and sort mirrors by region match and priority
        let mut matching_mirrors: Vec<_> = mirrors
            .iter()
            .filter(|m| m.enabled)
            .filter(|m| {
                m.region
                    .as_deref()
                    .map(|r| r == detected_region)
                    .unwrap_or(false)
            })
            .collect();

        // Sort by priority (higher = preferred)
        matching_mirrors.sort_by_key(|b| std::cmp::Reverse(b.priority));

        let mut urls = Vec::new();

        // Try to construct mirror URLs
        for mirror in &matching_mirrors {
            match self
                .download_url_for_mirror(&mirror.url, version, platform)
                .await
            {
                Ok(Some(mirror_url)) => {
                    tracing::debug!(
                        mirror = mirror.name,
                        region = ?mirror.region,
                        url = %mirror_url,
                        "Added mirror URL to download chain"
                    );
                    urls.push(mirror_url);
                }
                Ok(None) => {
                    tracing::debug!(
                        mirror = mirror.name,
                        "Mirror does not support URL construction for this runtime"
                    );
                }
                Err(e) => {
                    tracing::debug!(
                        mirror = mirror.name,
                        error = %e,
                        "Failed to construct mirror URL"
                    );
                }
            }
        }

        // Always include original URL as fallback
        urls.push(original_url.to_string());

        urls
    }

    /// Uninstall a specific version
    async fn uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        // New layout: uninstall from version dir directly; also clean up old platform dir.
        let platform = Platform::current();
        let base_path = ctx.paths.version_store_dir(self.store_name(), version);
        let platform_dir = base_path.join(platform.as_str());
        // Remove old platform-specific directory if it exists
        if ctx.fs.exists(&platform_dir) {
            ctx.fs.remove_dir_all(&platform_dir)?;
        }
        // Remove the version directory if it exists (new layout)
        if ctx.fs.exists(&base_path) {
            ctx.fs.remove_dir_all(&base_path)?;
        }
        Ok(())
    }

    /// Resolve version string to actual version
    ///
    /// This method resolves version requests like:
    /// - "latest" -> latest stable version
    /// - "3.11" -> latest 3.11.x version
    /// - "20" -> latest 20.x.x version
    /// - ">=3.9,<3.12" -> latest version in range
    /// - "^1.0.0" -> latest compatible version
    /// - "~1.0.0" -> latest patch version
    /// - "3.11.*" -> latest 3.11.x version
    async fn resolve_version(&self, version: &str, ctx: &RuntimeContext) -> Result<String> {
        let versions = self.fetch_versions(ctx).await?;

        let resolver = VersionResolver::new();
        let ecosystem = self.ecosystem();

        if let Some(resolved) = resolver.resolve(version, &versions, &ecosystem) {
            return Ok(resolved);
        }

        // Rust ecosystem passthrough: The Rust provider fetches *rustup* versions
        // (1.16–1.29), but users/tools often specify *rustc* versions (e.g. 1.93.1
        // from rust-toolchain.toml). When the requested version doesn't match any
        // rustup version, fall back to the latest rustup version and let rustup
        // handle the toolchain via --default-toolchain.
        if ecosystem == Ecosystem::Rust {
            let latest = versions
                .iter()
                .filter(|v| !v.prerelease)
                .map(|v| &v.version)
                .max_by(|a, b| {
                    let parse = |v: &str| -> Vec<u64> {
                        v.split('.').filter_map(|p| p.parse().ok()).collect()
                    };
                    parse(a).cmp(&parse(b))
                });

            if let Some(latest_version) = latest {
                tracing::info!(
                    "Rust passthrough: '{}' is not a rustup version, using latest rustup {} \
                     (rustup will manage the toolchain)",
                    version,
                    latest_version
                );
                return Ok(latest_version.clone());
            }
        }

        // Build a helpful error message with available version range
        let stable_versions: Vec<_> = versions
            .iter()
            .filter(|v| !v.prerelease)
            .map(|v| &v.version)
            .collect();

        let hint = if stable_versions.is_empty() {
            "No stable versions available.".to_string()
        } else {
            let min = stable_versions.last().map(|v| v.as_str()).unwrap_or("?");
            let max = stable_versions.first().map(|v| v.as_str()).unwrap_or("?");
            format!("Available versions: {} to {}", min, max)
        };

        Err(anyhow::anyhow!(
            "No version found for {} matching '{}'. {}",
            self.name(),
            version,
            hint
        ))
    }

    // ========== Shell Support (RFC 0038) ==========

    /// Get the path to a shell executable provided by this runtime
    ///
    /// This method is used when launching a shell with the runtime's environment
    /// (e.g., `vx git::git-bash`).
    ///
    /// # Arguments
    ///
    /// * `shell_name` - The name of the shell (e.g., "git-bash", "cmd", "bash")
    /// * `version` - The version of the runtime
    /// * `ctx` - Runtime context providing paths
    ///
    /// # Returns
    ///
    /// * `Some(path)` - The path to the shell executable
    /// * `None` - This runtime doesn't provide this shell
    ///
    /// # Example
    ///
    /// For Git, this would return the path to `git-bash.exe`:
    /// ```rust,ignore
    /// fn get_shell_path(&self, shell_name: &str, version: &str, ctx: &RuntimeContext) -> Option<PathBuf> {
    ///     if shell_name == "git-bash" {
    ///         let install_path = ctx.paths.version_store_dir("git", version);
    ///         Some(install_path.join("git-bash.exe"))
    ///     } else {
    ///         None
    ///     }
    /// }
    /// ```
    fn get_shell_path(
        &self,
        _shell_name: &str,
        _version: &str,
        _ctx: &RuntimeContext,
    ) -> Option<std::path::PathBuf> {
        None // Default: runtime doesn't provide any shells
    }

    /// Get list of shells provided by this runtime
    ///
    /// Returns a list of shell names that this runtime can provide.
    /// For example, Git provides ["git-bash", "git-cmd"].
    fn provided_shells(&self) -> Vec<&'static str> {
        vec![] // Default: no shells provided
    }
}
