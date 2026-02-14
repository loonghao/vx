//! Runtime trait definition
//!
//! The `Runtime` trait is the core abstraction for executable runtimes in vx.
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

use crate::context::{ExecutionContext, RuntimeContext};
use crate::ecosystem::Ecosystem;
use crate::platform::{Os, Platform};
use crate::region;
use crate::types::{ExecutionPrep, ExecutionResult, InstallResult, RuntimeDependency, VersionInfo};
use crate::version_resolver::VersionResolver;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

/// Detect the download region for mirror selection
///
/// Returns the region string used in provider.toml mirror configs (e.g., "cn", "global")
fn detect_download_region() -> &'static str {
    match region::detect_region() {
        region::Region::China => "cn",
        region::Region::Global => "global",
    }
}

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
    ///
    /// Used for tools installed via system package managers (winget, brew, apt, etc.)
    /// where the executable is available in the system PATH rather than a specific install path.
    pub fn success_system_installed() -> Self {
        Self {
            valid: true,
            executable_path: None, // System-installed, no specific path
            issues: vec![],
            suggestions: vec![],
        }
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
    fn mirror_urls(&self) -> Vec<vx_manifest::MirrorConfig> {
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
    fn normalize_config(&self) -> Option<&vx_manifest::NormalizeConfig> {
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
    fn post_extract(&self, _version: &str, _install_path: &std::path::PathBuf) -> Result<()> {
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

    /// Verify that an installation is valid and complete
    ///
    /// This method checks:
    /// 1. The executable exists at the expected path
    /// 2. The executable is actually executable (has correct permissions)
    /// 3. Any other runtime-specific requirements
    ///
    /// Override this for custom verification logic.
    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_relative = self.executable_relative_path(version, platform);

        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // Handle glob patterns in executable path (e.g., "*/bin/java.exe")
        let exe_path = if exe_relative.contains('*') {
            let pattern = install_path.join(&exe_relative);
            let pattern_str = pattern.to_string_lossy();
            match glob::glob(&pattern_str) {
                Ok(paths) => {
                    let matches: Vec<_> = paths.filter_map(|p| p.ok()).collect();
                    if matches.is_empty() {
                        None
                    } else {
                        Some(matches[0].clone())
                    }
                }
                Err(_) => None,
            }
        } else {
            let path = install_path.join(&exe_relative);
            if path.exists() { Some(path) } else { None }
        };

        // Check if executable exists
        let exe_path = match exe_path {
            Some(path) if path.exists() => path,
            _ => {
                issues.push(format!(
                    "Executable not found at expected path: {}",
                    install_path.join(&exe_relative).display()
                ));

                // Try to find the executable in the install directory
                if let Some(found_path) = self.find_executable_in_install_dir(install_path) {
                    suggestions.push(format!(
                        "Found executable at: {}. Consider overriding executable_relative_path() \
                         to return the correct relative path.",
                        found_path.display()
                    ));

                    // Calculate what the relative path should be
                    if let Ok(relative) = found_path.strip_prefix(install_path) {
                        suggestions.push(format!(
                            "Suggested executable_relative_path: \"{}\"",
                            relative.display()
                        ));
                    }
                } else {
                    // List top-level contents for debugging
                    if let Ok(entries) = std::fs::read_dir(install_path) {
                        let contents: Vec<_> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| {
                                let path = e.path();
                                let is_dir = path.is_dir();
                                format!(
                                    "{}{}",
                                    e.file_name().to_string_lossy(),
                                    if is_dir { "/" } else { "" }
                                )
                            })
                            .collect();
                        suggestions.push(format!(
                            "Install directory contents: [{}]",
                            contents.join(", ")
                        ));
                    }
                }

                return VerificationResult::failure(issues, suggestions);
            }
        };

        // Check if file is executable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&exe_path) {
                let mode = metadata.permissions().mode();
                if mode & 0o111 == 0 {
                    issues.push(format!(
                        "File exists but is not executable: {}",
                        exe_path.display()
                    ));
                    suggestions.push("Try: chmod +x <path>".to_string());
                    return VerificationResult::failure(issues, suggestions);
                }
            }
        }

        VerificationResult::success(exe_path)
    }

    /// Helper to find executable in install directory (searches up to 3 levels deep)
    fn find_executable_in_install_dir(&self, install_path: &Path) -> Option<std::path::PathBuf> {
        let platform = Platform::current();
        let exe_names =
            platform.all_executable_names(self.executable_name(), self.executable_extensions());

        for exe_name in &exe_names {
            if let Some(path) = self.search_for_executable(install_path, exe_name, 0, 3) {
                return Some(path);
            }
        }
        None
    }

    /// Recursively search for an executable
    fn search_for_executable(
        &self,
        dir: &Path,
        exe_name: &str,
        current_depth: usize,
        max_depth: usize,
    ) -> Option<std::path::PathBuf> {
        if current_depth > max_depth || !dir.exists() {
            return None;
        }

        let entries = std::fs::read_dir(dir).ok()?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Check for exact match or match without extension
                    if name == exe_name || name == self.name() {
                        return Some(path);
                    }
                }
            } else if path.is_dir()
                && let Some(found) =
                    self.search_for_executable(&path, exe_name, current_depth + 1, max_depth)
            {
                return Some(found);
            }
        }

        None
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

        // Fail early with a clear message when the provider doesn't support this platform.
        if let Err(msg) = self.check_platform_support() {
            return Err(anyhow::anyhow!(msg));
        }

        // Use store_name() which handles aliases and bundled runtimes
        // e.g., "npm" -> "node", "uvx" -> "uv", "vscode" -> "code"
        let store_name = self.store_name();
        let platform = Platform::current();
        let exe_relative = self.executable_relative_path(version, &platform);

        // Get base version directory, then append platform-specific subdirectory
        // This implements platform redirection: <provider>/<version>/<platform>/
        let base_install_path = ctx.paths.version_store_dir(store_name, version);
        let install_path = base_install_path.join(platform.as_str());

        debug!(
            "Install path for {} (store: {}) {}: {} (platform: {})",
            self.name(),
            store_name,
            version,
            install_path.display(),
            platform.as_str()
        );
        debug!("Executable relative path: {}", exe_relative);

        // Check if already installed
        if ctx.fs.exists(&install_path) {
            // Use verify_installation to check if the executable exists (supports glob patterns)
            let verification = self.verify_installation(version, &install_path, &platform);
            if verification.valid {
                let exe_path = verification
                    .executable_path
                    .unwrap_or_else(|| install_path.join(&exe_relative));
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

        // Try to use cached URL from lock file first
        // Try tool_name first, then store_name (for bundled tools like npm -> node)
        // NOTE: Cached URLs may be platform-specific (e.g., from vx.lock generated on
        // a different OS). We validate that the cached URL is plausible for the current
        // platform before using it.
        let url = if let Some(cached_url) = ctx
            .get_cached_download_url(self.name())
            .filter(|u| is_url_plausible_for_platform(u, &platform))
        {
            debug!(
                "Using cached download URL from lock file for {}: {}",
                self.name(),
                cached_url
            );
            cached_url
        } else if let Some(cached_url) = ctx
            .get_cached_download_url(self.store_name())
            .filter(|u| is_url_plausible_for_platform(u, &platform))
        {
            debug!(
                "Using cached download URL from lock file for {}: {}",
                self.store_name(),
                cached_url
            );
            cached_url
        } else {
            // Fall back to runtime's download_url method
            self.download_url(version, &platform)
                .await?
                .ok_or_else(|| anyhow::anyhow!("No download URL for {} {}", self.name(), version))?
        };

        // Build mirror URL list: try region-matching mirrors first, then original URL
        let download_urls = self
            .build_download_url_chain(&url, version, &platform)
            .await;

        info!(
            "Downloading {} {} ({})",
            self.name(),
            version,
            if download_urls.len() > 1 {
                format!("{} sources available", download_urls.len())
            } else {
                "direct".to_string()
            }
        );

        // Prepare layout metadata if available
        let mut layout_metadata = std::collections::HashMap::new();
        if let Some(layout) = self.executable_layout() {
            use crate::layout::LayoutContext;

            let layout_ctx = LayoutContext {
                version: version.to_string(),
                name: self.name().to_string(),
                platform: platform.clone(),
            };

            if let Ok(resolved) = layout.resolve(&layout_ctx) {
                match resolved {
                    crate::layout::ResolvedLayout::Binary {
                        source_name,
                        target_name,
                        target_dir,
                        permissions,
                    } => {
                        layout_metadata.insert("source_name".to_string(), source_name);
                        layout_metadata.insert("target_name".to_string(), target_name);
                        layout_metadata.insert("target_dir".to_string(), target_dir);
                        if let Some(perms) = permissions {
                            layout_metadata.insert("target_permissions".to_string(), perms);
                        }
                    }
                    crate::layout::ResolvedLayout::Archive {
                        strip_prefix,
                        permissions,
                        ..
                    } => {
                        // Pass strip_prefix to installer for archive extraction
                        if let Some(prefix) = strip_prefix {
                            layout_metadata.insert("strip_prefix".to_string(), prefix);
                        }
                        if let Some(perms) = permissions {
                            layout_metadata.insert("target_permissions".to_string(), perms);
                        }
                    }
                }
            }
        }

        // Download with mirror fallback chain
        let mut last_error = None;
        for (i, download_url) in download_urls.iter().enumerate() {
            let is_mirror = i < download_urls.len() - 1 || download_urls.len() == 1;
            if i > 0 {
                info!(
                    "Mirror failed, trying {} (source {}/{})",
                    download_url,
                    i + 1,
                    download_urls.len()
                );
                // Clean up failed partial download
                if ctx.fs.exists(&install_path)
                    && let Err(e) = std::fs::remove_dir_all(&install_path)
                {
                    debug!("Failed to clean up partial download: {}", e);
                }
            } else {
                info!("Downloading from {}", download_url);
            }

            let result = if !layout_metadata.is_empty() {
                ctx.installer
                    .download_with_layout(download_url, &install_path, &layout_metadata)
                    .await
            } else {
                ctx.installer
                    .download_and_extract(download_url, &install_path)
                    .await
            };

            match result {
                Ok(()) => {
                    if i > 0 {
                        info!(
                            "Successfully downloaded {} {} from fallback source",
                            self.name(),
                            version
                        );
                    }
                    last_error = None;
                    break;
                }
                Err(e) => {
                    if is_mirror && download_urls.len() > 1 {
                        debug!(
                            "Download from {} failed: {}, will try next source",
                            download_url, e
                        );
                    }
                    last_error = Some(e);
                }
            }
        }

        if let Some(err) = last_error {
            return Err(err);
        }

        // Run post-extract hook
        self.post_extract(version, &install_path)?;

        // RFC 0022: Post-install normalization
        if let Some(normalize_config) = self.normalize_config() {
            use crate::normalizer::{NormalizeContext, Normalizer};

            let normalize_ctx = NormalizeContext::new(self.name(), version);
            match Normalizer::normalize(&install_path, normalize_config, &normalize_ctx) {
                Ok(result) => {
                    if result.has_changes() {
                        debug!("Normalization completed: {}", result.summary());
                    }
                }
                Err(e) => {
                    // Normalization errors are warnings, not failures
                    debug!("Normalization warning: {}", e);
                }
            }
        }

        debug!("Expected executable path pattern: {}", exe_relative);

        // Use the verification framework to check installation
        let verification = self.verify_installation(version, &install_path, &platform);

        if !verification.valid {
            // Build a detailed error message
            let mut error_msg = format!(
                "Installation of {} {} failed verification.\n",
                self.name(),
                version
            );

            error_msg.push_str("\nIssues found:\n");
            for issue in &verification.issues {
                error_msg.push_str(&format!("  - {}\n", issue));
            }

            if !verification.suggestions.is_empty() {
                error_msg.push_str("\nSuggestions:\n");
                for suggestion in &verification.suggestions {
                    error_msg.push_str(&format!("  - {}\n", suggestion));
                }
            }

            return Err(anyhow::anyhow!(error_msg));
        }

        let verified_exe_path = verification
            .executable_path
            .unwrap_or_else(|| install_path.join(&exe_relative));

        Ok(InstallResult::success(
            install_path,
            verified_exe_path,
            version.to_string(),
        ))
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

        // Use platform-specific directory for installation check
        let platform = Platform::current();
        let base_path = ctx.paths.version_store_dir(self.store_name(), version);
        let install_path = base_path.join(platform.as_str());
        Ok(ctx.fs.exists(&install_path))
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
        // Use platform-specific directory for executable lookup
        let platform = Platform::current();
        let base_path = ctx.paths.version_store_dir(self.store_name(), version);
        let install_path = base_path.join(platform.as_str());
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
    /// See: https://github.com/loonghao/vx/issues/573
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
        matching_mirrors.sort_by(|a, b| b.priority.cmp(&a.priority));

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
        // Use platform-specific directory for uninstallation
        let platform = Platform::current();
        let base_path = ctx.paths.version_store_dir(self.store_name(), version);
        let install_path = base_path.join(platform.as_str());
        if ctx.fs.exists(&install_path) {
            ctx.fs.remove_dir_all(&install_path)?;
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

        resolver
            .resolve(version, &versions, &ecosystem)
            .ok_or_else(|| {
                // Build a helpful error message with available version range
                let stable_versions: Vec<_> = versions
                    .iter()
                    .filter(|v| !v.prerelease)
                    .map(|v| &v.version)
                    .collect();

                let hint = if stable_versions.is_empty() {
                    "No stable versions available.".to_string()
                } else {
                    // Get min and max versions
                    let min = stable_versions.last().map(|v| v.as_str()).unwrap_or("?");
                    let max = stable_versions.first().map(|v| v.as_str()).unwrap_or("?");
                    format!("Available versions: {} to {}", min, max)
                };

                anyhow::anyhow!(
                    "No version found for {} matching '{}'. {}",
                    self.name(),
                    version,
                    hint
                )
            })
    }
}

/// Check if a download URL is plausible for the given platform.
///
/// This is a defensive check to avoid using a cached URL that was recorded for
/// a different platform (e.g., a Windows `.zip` URL on Linux). We look for
/// platform-specific markers in the URL.
fn is_url_plausible_for_platform(url: &str, platform: &Platform) -> bool {
    let url_lower = url.to_lowercase();

    // Platform markers that indicate a URL is for a specific OS
    let windows_markers = ["windows", "win32", "win64", "-msvc", ".msi"];
    let macos_markers = ["darwin", "macos", "osx", ".dmg"];
    let linux_markers = ["linux", "gnu", "musl"];

    let url_is_windows = windows_markers.iter().any(|m| url_lower.contains(m));
    let url_is_macos = macos_markers.iter().any(|m| url_lower.contains(m));
    let url_is_linux = linux_markers.iter().any(|m| url_lower.contains(m));

    // If we can't detect any platform markers, assume the URL is universal
    if !url_is_windows && !url_is_macos && !url_is_linux {
        return true;
    }

    // Check that the URL's platform matches the current platform
    match platform.os {
        Os::Windows => url_is_windows,
        Os::MacOS => url_is_macos,
        Os::Linux => url_is_linux,
        _ => true, // Unknown platform — allow any URL
    }
}
