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
use crate::platform::Platform;
use crate::types::{ExecutionResult, InstallResult, RuntimeDependency, VersionInfo};
use crate::version_resolver::VersionResolver;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

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
            if path.exists() {
                Some(path)
            } else {
                None
            }
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
            } else if path.is_dir() {
                if let Some(found) =
                    self.search_for_executable(&path, exe_name, current_depth + 1, max_depth)
                {
                    return Some(found);
                }
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
        let install_path = ctx.paths.version_store_dir(store_name, version);
        let platform = Platform::current();
        let exe_relative = self.executable_relative_path(version, &platform);

        debug!(
            "Install path for {} (store: {}) {}: {}",
            self.name(),
            store_name,
            version,
            install_path.display()
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

        let url = self
            .download_url(version, &platform)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No download URL for {} {}", self.name(), version))?;

        info!("Downloading {} {} from {}", self.name(), version, url);

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

        // Download and extract (with layout support if metadata is provided)
        if !layout_metadata.is_empty() {
            ctx.installer
                .download_with_layout(&url, &install_path, &layout_metadata)
                .await?;
        } else {
            ctx.installer
                .download_and_extract(&url, &install_path)
                .await?;
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
    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        let install_path = ctx.paths.version_store_dir(self.store_name(), version);
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
        let install_path = ctx.paths.version_store_dir(self.store_name(), version);
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

    /// Get download URL for a specific version and platform
    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Default implementation - subclasses should override
        let _ = (version, platform);
        Ok(None)
    }

    /// Uninstall a specific version
    async fn uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let install_path = ctx.paths.version_store_dir(self.store_name(), version);
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
