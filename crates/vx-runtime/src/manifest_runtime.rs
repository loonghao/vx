//! Manifest-Driven Runtime implementation
//!
//! This module provides a Runtime implementation that is driven entirely by
//! provider.toml configuration files. It's designed for system tools that
//! don't require strict version management (git, cmake, curl, etc.).
//!
//! # Design Goals
//!
//! 1. **Zero Rust code for system tools** - All configuration in TOML
//! 2. **User-extensible** - Users can add their own tools via ~/.vx/providers/
//! 3. **System package manager integration** - Leverage choco, winget, brew, apt
//! 4. **Fallback strategies** - Multiple installation methods with priorities

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, info, warn};

use crate::{Ecosystem, InstallResult, Platform, Runtime, RuntimeContext, VersionInfo};
use vx_runtime_core::{MirrorConfig, NormalizeConfig};
use vx_system_pm::{PackageInstallSpec, PackageManagerRegistry};

/// Source of a provider
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderSource {
    /// Built-in provider (compiled into vx)
    BuiltIn,
    /// User local provider (~/.vx/providers/)
    UserLocal(PathBuf),
    /// Environment variable specified path
    EnvPath(PathBuf),
}

impl std::fmt::Display for ProviderSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderSource::BuiltIn => write!(f, "built-in"),
            ProviderSource::UserLocal(p) => write!(f, "{}", p.display()),
            ProviderSource::EnvPath(p) => write!(f, "{} (env)", p.display()),
        }
    }
}

/// Type alias for an async fetch_versions function injected from Starlark providers.
///
/// This allows `ManifestDrivenRuntime` to delegate `fetch_versions` to a
/// Starlark-driven implementation (e.g. from `provider.star`) without creating
/// a circular dependency between `vx-runtime` and `vx-starlark`.
pub type FetchVersionsFn = Arc<
    dyn Fn()
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionInfo>>> + Send>>
        + Send
        + Sync,
>;

/// Type alias for an async download_url function injected from Starlark providers.
///
/// Signature: `(version: String) -> Option<String>`
pub type DownloadUrlFn = Arc<
    dyn Fn(
            String,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<String>>> + Send>>
        + Send
        + Sync,
>;

/// Type alias for an async install_layout function injected from Starlark providers.
///
/// Returns a serialized JSON value describing the install layout, or `None`.
pub type InstallLayoutFn = Arc<
    dyn Fn(
            String,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<serde_json::Value>>> + Send>,
        > + Send
        + Sync,
>;

/// A runtime driven by manifest configuration (provider.toml)
///
/// This is used for system tools that don't require strict version management.
/// The runtime is entirely configured via TOML, with no Rust code needed.
///
/// For Starlark-driven providers (those with a `provider.star` that defines
/// `fetch_versions`), the `fetch_versions_fn` field can be set to delegate
/// version fetching to the Starlark engine.  This avoids a circular dependency
/// between `vx-runtime` and `vx-starlark`.
#[derive(Clone)]
pub struct ManifestDrivenRuntime {
    /// Runtime name
    pub name: String,
    /// Description
    pub description: String,
    /// Executable name
    pub executable: String,
    /// Aliases
    pub aliases: Vec<String>,
    /// Ecosystem (e.g. NodeJs, Python, Go)
    pub ecosystem_override: Option<Ecosystem>,
    /// If bundled with another runtime (affects store_name)
    pub bundled_with: Option<String>,
    /// Provider name
    pub provider_name: String,
    /// Provider source
    pub source: ProviderSource,
    /// System installation strategies
    pub install_strategies: Vec<InstallStrategy>,
    /// Tools provided by this runtime
    pub provides: Vec<ProvidedTool>,
    /// Detection configuration
    pub detection: Option<DetectionConfig>,
    /// System dependencies
    pub system_deps: Option<SystemDepsConfig>,
    /// Post-install normalize configuration (RFC 0022)
    pub normalize: Option<NormalizeConfig>,
    /// Mirror configurations from provider.toml (RFC 0018)
    pub mirrors: Vec<MirrorConfig>,
    /// Optional Starlark-driven fetch_versions implementation.
    ///
    /// When set, `fetch_versions()` delegates to this function instead of
    /// returning the default `["system"]` placeholder.  Injected by
    /// provider crates that have a `provider.star` with `fetch_versions`.
    pub fetch_versions_fn: Option<FetchVersionsFn>,

    /// Optional Starlark-driven download_url implementation.
    ///
    /// When set, `download_url()` delegates to this function instead of
    /// scanning `install_strategies` for a `DirectDownload` entry.
    pub download_url_fn: Option<DownloadUrlFn>,

    /// Optional Starlark-driven install_layout implementation.
    ///
    /// When set, the `install()` method uses the returned layout descriptor
    /// (strip_prefix, executable_paths, etc.) instead of the default logic.
    pub install_layout_fn: Option<InstallLayoutFn>,

    /// Optional pip package name.
    ///
    /// When set, `install()` will use `uv pip install <pip_package>==<version>`
    /// to install this runtime, and `fetch_versions()` will query PyPI.
    /// Used for Python tools like meson, black, ruff, etc.
    pub pip_package: Option<String>,

    /// Shells provided by this runtime (RFC 0038)
    ///
    /// Maps shell names to their relative paths from the install directory.
    /// For example, Git provides:
    /// - "git-bash" -> "git-bash.exe"
    /// - "git-cmd" -> "git-cmd.exe"
    pub shells: Vec<ShellDefinition>,
    /// Platform OS constraint from provider.star `platforms = {"os": [...]}`.
    ///
    /// When non-empty, `supported_platforms()` returns only platforms whose
    /// OS name matches one of the entries (e.g. `["macos"]` → macOS-only).
    /// An empty vec means "all platforms" (no constraint).
    pub platform_os: Vec<String>,
}

/// Shell definition for runtime-provided shells (RFC 0038)
#[derive(Debug, Clone)]
pub struct ShellDefinition {
    /// Shell name (e.g., "git-bash", "cmd")
    pub name: String,
    /// Relative path from install directory (e.g., "git-bash.exe", "bin/bash.exe")
    pub path: String,
}

/// Installation strategy for system tools
#[derive(Debug, Clone)]
pub enum InstallStrategy {
    /// Use a system package manager
    PackageManager {
        /// Package manager name (choco, winget, brew, apt, etc.)
        manager: String,
        /// Package name
        package: String,
        /// Installation parameters (Chocolatey --params)
        params: Option<String>,
        /// Native installer arguments
        install_args: Option<String>,
        /// Priority (higher = preferred)
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
    /// Direct download
    DirectDownload {
        /// URL template (supports {version}, {platform}, {arch})
        url: String,
        /// Archive format (zip, tar.gz, etc.)
        format: Option<String>,
        /// Executable path within archive
        executable_path: Option<String>,
        /// Priority
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
    /// Run a script
    Script {
        /// Script URL
        url: String,
        /// Script type (powershell, bash, cmd)
        script_type: ScriptType,
        /// Script arguments
        args: Vec<String>,
        /// Priority
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
    /// Provided by another runtime
    ProvidedBy {
        /// Provider runtime name
        provider: String,
        /// Relative path to executable
        relative_path: String,
        /// Priority
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
}

impl InstallStrategy {
    /// Get the priority of this strategy
    pub fn priority(&self) -> i32 {
        match self {
            InstallStrategy::PackageManager { priority, .. } => *priority,
            InstallStrategy::DirectDownload { priority, .. } => *priority,
            InstallStrategy::Script { priority, .. } => *priority,
            InstallStrategy::ProvidedBy { priority, .. } => *priority,
        }
    }

    /// Check if this strategy matches the current platform
    pub fn matches_platform(&self, platform: &Platform) -> bool {
        let platforms = match self {
            InstallStrategy::PackageManager { platforms, .. } => platforms,
            InstallStrategy::DirectDownload { platforms, .. } => platforms,
            InstallStrategy::Script { platforms, .. } => platforms,
            InstallStrategy::ProvidedBy { platforms, .. } => platforms,
        };

        if platforms.is_empty() {
            return true; // No filter = all platforms
        }

        let current = platform.os_name();
        platforms.iter().any(|p| p.eq_ignore_ascii_case(current))
    }
}

/// Script type for installation
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptType {
    PowerShell,
    Bash,
    Cmd,
}

/// Tool provided by a runtime
#[derive(Debug, Clone)]
pub struct ProvidedTool {
    /// Tool name
    pub name: String,
    /// Relative path to executable
    pub relative_path: String,
    /// Supported platforms
    pub platforms: Vec<String>,
}

/// Detection configuration for version detection
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Command to run (e.g., "{executable} --version")
    pub command: String,
    /// Regex pattern to extract version
    pub pattern: String,
    /// System paths to search
    pub system_paths: Vec<String>,
    /// Environment variable hints
    pub env_hints: Vec<String>,
}

/// System dependencies configuration
#[derive(Debug, Clone, Default)]
pub struct SystemDepsConfig {
    /// Pre-installation dependencies
    pub pre_depends: Vec<SystemDependency>,
    /// Runtime dependencies
    pub depends: Vec<SystemDependency>,
    /// Recommended dependencies
    pub recommends: Vec<SystemDependency>,
    /// Optional dependencies
    pub suggests: Vec<SystemDependency>,
}

/// A system-level dependency
#[derive(Debug, Clone)]
pub struct SystemDependency {
    /// Dependency type
    pub dep_type: SystemDepType,
    /// Dependency identifier
    pub id: String,
    /// Version constraint
    pub version: Option<String>,
    /// Reason for dependency
    pub reason: Option<String>,
    /// Platform filter
    pub platforms: Vec<String>,
    /// Whether this is optional
    pub optional: bool,
}

/// Type of system dependency
#[derive(Debug, Clone, PartialEq)]
pub enum SystemDepType {
    /// Windows KB update
    WindowsKb,
    /// Windows Feature (DISM)
    WindowsFeature,
    /// Visual C++ Redistributable
    VcRedist,
    /// .NET Framework / Runtime
    DotNet,
    /// System package
    Package,
    /// Another vx runtime
    Runtime,
}

impl std::fmt::Debug for ManifestDrivenRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManifestDrivenRuntime")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("executable", &self.executable)
            .field("aliases", &self.aliases)
            .field("provider_name", &self.provider_name)
            .field(
                "fetch_versions_fn",
                &self.fetch_versions_fn.as_ref().map(|_| "<fn>"),
            )
            .finish()
    }
}

impl ManifestDrivenRuntime {
    /// Create a new manifest-driven runtime
    pub fn new(
        name: impl Into<String>,
        provider_name: impl Into<String>,
        source: ProviderSource,
    ) -> Self {
        let name = name.into();
        Self {
            executable: name.clone(),
            name,
            description: String::new(),
            aliases: Vec::new(),
            ecosystem_override: None,
            bundled_with: None,
            provider_name: provider_name.into(),
            source,
            install_strategies: Vec::new(),
            provides: Vec::new(),
            detection: None,
            system_deps: None,
            normalize: None,
            mirrors: Vec::new(),
            fetch_versions_fn: None,
            download_url_fn: None,
            install_layout_fn: None,
            pip_package: None,
            shells: Vec::new(),
            platform_os: Vec::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set mirrors from provider.toml configuration
    pub fn with_mirrors(mut self, mirrors: Vec<MirrorConfig>) -> Self {
        self.mirrors = mirrors;
        self
    }

    /// Set bundled_with (parent runtime for bundled tools)
    pub fn with_bundled_with(mut self, bundled_with: impl Into<String>) -> Self {
        self.bundled_with = Some(bundled_with.into());
        self
    }

    /// Set platform OS constraint (e.g. `["macos"]` for macOS-only tools).
    ///
    /// When set, `supported_platforms()` will only return platforms whose OS
    /// matches one of the provided names.  Accepted values: `"windows"`,
    /// `"macos"`, `"linux"`, `"freebsd"`.
    pub fn with_platform_os(mut self, platform_os: Vec<String>) -> Self {
        self.platform_os = platform_os;
        self
    }

    /// Set executable name
    pub fn with_executable(mut self, executable: impl Into<String>) -> Self {
        self.executable = executable.into();
        self
    }

    /// Add an alias
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add multiple aliases from a Vec<String>
    pub fn with_aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases.extend(aliases);
        self
    }

    /// Set the ecosystem for this runtime
    pub fn with_ecosystem(mut self, ecosystem: Ecosystem) -> Self {
        self.ecosystem_override = Some(ecosystem);
        self
    }

    /// Add an installation strategy
    pub fn with_strategy(mut self, strategy: InstallStrategy) -> Self {
        self.install_strategies.push(strategy);
        self
    }

    /// Set the pip package name for Python-based tools.
    ///
    /// When set, `install()` uses `uv pip install <package>==<version>` and
    /// `fetch_versions()` queries PyPI for available versions.
    pub fn with_pip_package(mut self, package: impl Into<String>) -> Self {
        self.pip_package = Some(package.into());
        self
    }

    /// Set shells provided by this runtime (RFC 0038)
    ///
    /// Shells are executables that can be launched with the runtime's environment.
    /// For example, Git provides "git-bash" and "git-cmd".
    pub fn with_shells(mut self, shells: Vec<ShellDefinition>) -> Self {
        self.shells = shells;
        self
    }

    /// Add a single shell definition
    pub fn with_shell(mut self, name: impl Into<String>, path: impl Into<String>) -> Self {
        self.shells.push(ShellDefinition {
            name: name.into(),
            path: path.into(),
        });
        self
    }

    /// Set detection configuration
    pub fn with_detection(mut self, detection: DetectionConfig) -> Self {
        self.detection = Some(detection);
        self
    }

    /// Set normalize configuration
    pub fn with_normalize(mut self, normalize: NormalizeConfig) -> Self {
        self.normalize = Some(normalize);
        self
    }

    /// Set install dependencies (vx-managed runtimes that must be installed first)
    ///
    /// This is a convenience method that converts a list of runtime names
    /// (with optional version constraints) into a `SystemDepsConfig`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In provider.star:
    /// // install_deps = ["7zip", "node>=18"]
    /// runtime = runtime.with_install_deps(vec!["7zip".to_string(), "node>=18".to_string()]);
    /// ```
    pub fn with_install_deps(mut self, deps: Vec<String>) -> Self {
        if deps.is_empty() {
            return self;
        }

        let pre_depends: Vec<SystemDependency> = deps
            .into_iter()
            .map(|dep| {
                // Parse "name>=version" or "name" format
                let (id, version) = if let Some(gt_pos) = dep.find(">=") {
                    (
                        dep[..gt_pos].to_string(),
                        Some(dep[gt_pos + 2..].to_string()),
                    )
                } else if let Some(eq_pos) = dep.find('=') {
                    (
                        dep[..eq_pos].to_string(),
                        Some(dep[eq_pos + 1..].to_string()),
                    )
                } else {
                    (dep, None)
                };

                SystemDependency {
                    dep_type: SystemDepType::Runtime,
                    id,
                    version,
                    reason: Some("Install dependency".to_string()),
                    platforms: vec![],
                    optional: false,
                }
            })
            .collect();

        self.system_deps = Some(SystemDepsConfig {
            pre_depends,
            depends: vec![],
            recommends: vec![],
            suggests: vec![],
        });
        self
    }

    /// Inject a Starlark-driven `fetch_versions` implementation.
    ///
    /// Call this from provider crates that have a `provider.star` with a
    /// `fetch_versions` function.  When set, `Runtime::fetch_versions()` will
    /// delegate to this closure instead of returning the `["system"]` fallback.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use vx_runtime::{ManifestDrivenRuntime, ProviderSource};
    ///
    /// let runtime = ManifestDrivenRuntime::new("just", "just", ProviderSource::BuiltIn)
    ///     .with_fetch_versions(|| {
    ///         Box::pin(async {
    ///             // Call StarlarkProvider::fetch_versions() here
    ///             Ok(vec![])
    ///         })
    ///     });
    /// ```
    pub fn with_fetch_versions<F>(mut self, f: F) -> Self
    where
        F: Fn() -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<Vec<VersionInfo>>> + Send>,
            > + Send
            + Sync
            + 'static,
    {
        self.fetch_versions_fn = Some(Arc::new(f));
        self
    }

    /// Inject a Starlark-driven `download_url` implementation.
    ///
    /// When set, `Runtime::download_url()` delegates to this closure instead
    /// of scanning `install_strategies` for a `DirectDownload` entry.
    pub fn with_download_url<F>(mut self, f: F) -> Self
    where
        F: Fn(
                String,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<Option<String>>> + Send>,
            > + Send
            + Sync
            + 'static,
    {
        self.download_url_fn = Some(Arc::new(f));
        self
    }

    /// Inject a Starlark-driven `install_layout` implementation.
    ///
    /// When set, the `install()` method uses the returned layout descriptor
    /// to determine strip_prefix and executable_paths.
    pub fn with_install_layout<F>(mut self, f: F) -> Self
    where
        F: Fn(
                String,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<Option<serde_json::Value>>> + Send>,
            > + Send
            + Sync
            + 'static,
    {
        self.install_layout_fn = Some(Arc::new(f));
        self
    }

    /// Resolve the executable path from a Starlark install_layout descriptor.
    ///
    /// Tries `executable_paths` list first, then falls back to the runtime's
    /// own executable name.
    fn resolve_exe_path_from_layout(
        &self,
        install_dir: &std::path::Path,
        layout: &serde_json::Value,
    ) -> std::path::PathBuf {
        // Try each path in executable_paths
        if let Some(paths) = layout.get("executable_paths").and_then(|p| p.as_array()) {
            for p in paths {
                if let Some(rel) = p.as_str() {
                    let candidate = install_dir.join(rel);
                    if candidate.exists() {
                        return candidate;
                    }
                }
            }
            // Return first entry even if it doesn't exist yet
            if let Some(first) = paths.first().and_then(|p| p.as_str()) {
                return install_dir.join(first);
            }
        }
        // Fallback: use executable name
        install_dir.join(vx_paths::with_executable_extension(&self.executable))
    }

    /// Select the best available installation strategy
    pub async fn select_best_strategy(&self, platform: &Platform) -> Option<&InstallStrategy> {
        let mut candidates: Vec<_> = self
            .install_strategies
            .iter()
            .filter(|s| s.matches_platform(platform))
            .collect();

        // Sort by priority (descending)
        candidates.sort_by_key(|b| std::cmp::Reverse(b.priority()));

        // Return the first available strategy
        for strategy in candidates {
            if self.is_strategy_available(strategy).await {
                return Some(strategy);
            }
        }

        None
    }

    /// Check if a strategy is available on the current system
    async fn is_strategy_available(&self, strategy: &InstallStrategy) -> bool {
        match strategy {
            InstallStrategy::PackageManager { manager, .. } => {
                // Check if the package manager is installed
                is_package_manager_available(manager).await
            }
            InstallStrategy::DirectDownload { .. } => true,
            InstallStrategy::Script { .. } => true,
            InstallStrategy::ProvidedBy { provider, .. } => {
                // Check if the provider runtime is installed
                which::which(provider).is_ok()
            }
        }
    }

    /// Detect the installed version using the detection configuration
    pub async fn detect_version(&self) -> Result<Option<String>> {
        let detection = match &self.detection {
            Some(d) => d,
            None => return Ok(None),
        };

        // Find the executable
        let executable_path = match which::which(&self.executable) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        // Build the command
        let command = detection.command.replace("{executable}", &self.executable);
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        // Execute the command
        let output = tokio::process::Command::new(&executable_path)
            .args(&parts[1..])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(None);
        }

        // Parse the output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        // Extract version using regex
        let re = regex::Regex::new(&detection.pattern)?;
        if let Some(captures) = re.captures(&combined)
            && let Some(version) = captures.get(1)
        {
            return Ok(Some(version.as_str().to_string()));
        }

        Ok(None)
    }
}

#[async_trait]
impl Runtime for ManifestDrivenRuntime {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        if self.description.is_empty() {
            "System tool"
        } else {
            &self.description
        }
    }

    fn aliases(&self) -> &[&str] {
        // This is a limitation - we can't return borrowed slices from owned Vec
        // In practice, this method might need to be redesigned
        &[]
    }

    fn aliases_owned(&self) -> Vec<String> {
        self.aliases.clone()
    }

    fn ecosystem(&self) -> Ecosystem {
        self.ecosystem_override.clone().unwrap_or(Ecosystem::System)
    }

    fn supported_platforms(&self) -> Vec<crate::platform::Platform> {
        if self.platform_os.is_empty() {
            return crate::platform::Platform::all_common();
        }
        // Build platform list from the OS constraint strings
        let mut platforms = Vec::new();
        for os_name in &self.platform_os {
            match os_name.to_lowercase().as_str() {
                "windows" => platforms.extend(crate::platform::Platform::windows_only()),
                "macos" | "darwin" | "osx" => {
                    platforms.extend(crate::platform::Platform::macos_only())
                }
                "linux" => platforms.extend(crate::platform::Platform::linux_only()),
                "unix" => platforms.extend(crate::platform::Platform::unix_only()),
                _ => {}
            }
        }
        if platforms.is_empty() {
            crate::platform::Platform::all_common()
        } else {
            platforms
        }
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("provider".to_string(), self.provider_name.clone());
        meta.insert("source".to_string(), self.source.to_string());
        meta.insert("manifest_driven".to_string(), "true".to_string());
        if let Some(ref bundled) = self.bundled_with {
            meta.insert("bundled_with".to_string(), bundled.clone());
        }
        meta
    }

    fn mirror_urls(&self) -> Vec<MirrorConfig> {
        self.mirrors.clone()
    }

    /// Get the store directory name for this runtime
    ///
    /// For bundled runtimes, returns the parent runtime's name.
    /// For standalone runtimes, returns self.name.
    fn store_name(&self) -> &str {
        self.bundled_with.as_deref().unwrap_or(&self.name)
    }

    /// Check if this runtime version is directly installable
    ///
    /// For bundled runtimes (e.g., npm bundled with node), returns `false`
    /// because they are not installed separately - they come with the parent runtime.
    fn is_version_installable(&self, _version: &str) -> bool {
        self.bundled_with.is_none()
    }

    /// Prepare execution for bundled runtimes
    ///
    /// For bundled runtimes, we need to find the executable in the parent runtime's
    /// installation directory. The resolver should have already set up the correct
    /// store directory, so we just need to find the executable there.
    async fn prepare_execution(
        &self,
        version: &str,
        _ctx: &crate::ExecutionContext,
    ) -> Result<crate::ExecutionPrep> {
        // For bundled runtimes, find the executable in the parent's directory
        if let Some(ref parent) = self.bundled_with {
            debug!(
                "Preparing bundled runtime {} (bundled with {}) at version {}",
                self.name, parent, version
            );

            // Try to find the executable in the parent's store directory
            let paths = vx_paths::VxPaths::new()
                .map_err(|e| anyhow::anyhow!("Failed to get VxPaths: {}", e))?;
            let store_name = parent;
            let platform = crate::platform::Platform::current();
            let version_dir = paths.version_store_dir(store_name, version);

            // Try platform-specific directory first
            let platform_dir = version_dir.join(platform.as_str());

            // Search for the executable
            let exe_name = &self.executable;
            let exe_with_ext = if cfg!(windows) {
                if exe_name.ends_with(".exe")
                    || exe_name.ends_with(".cmd")
                    || exe_name.ends_with(".bat")
                {
                    exe_name.to_string()
                } else {
                    format!("{}.exe", exe_name)
                }
            } else {
                exe_name.clone()
            };

            let search_dirs = [&platform_dir, &version_dir];

            for dir in &search_dirs {
                // Check direct and bin/ subdirectory
                let candidates = [
                    dir.join(&exe_with_ext),
                    dir.join(exe_name),
                    dir.join("bin").join(&exe_with_ext),
                    dir.join("bin").join(exe_name),
                ];

                for path in candidates {
                    if path.exists() {
                        debug!(
                            "Found bundled executable {} at {}",
                            self.name,
                            path.display()
                        );
                        return Ok(crate::ExecutionPrep {
                            executable_override: Some(path),
                            proxy_ready: true,
                            message: Some(format!(
                                "Using {} from {} installation",
                                self.name, parent
                            )),
                            ..Default::default()
                        });
                    }
                }
            }

            // Not found - parent runtime may need to be installed
            warn!(
                "Could not find {} executable in {} installation. {} may need to be installed.",
                self.name, parent, parent
            );
            return Ok(crate::ExecutionPrep {
                use_system_path: true,
                message: Some(format!(
                    "{} not found in {} installation, trying system PATH",
                    self.name, parent
                )),
                ..Default::default()
            });
        }

        // For non-bundled runtimes, use default behavior
        Ok(crate::ExecutionPrep::default())
    }

    /// Fetch available versions.
    ///
    /// If a Starlark-driven `fetch_versions_fn` was injected via
    /// [`ManifestDrivenRuntime::with_fetch_versions`], delegates to that
    /// function (which calls the `fetch_versions` function in `provider.star`).
    ///
    /// Otherwise falls back to returning `["system"]`, indicating that this
    /// runtime is managed by the OS package manager and has no vx-managed
    /// version list.
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        if let Some(ref f) = self.fetch_versions_fn {
            return f().await;
        }
        // pip package: query PyPI for available versions
        if let Some(ref pkg) = self.pip_package {
            let url = format!("https://pypi.org/pypi/{}/json", pkg);
            if let Ok(resp) = ctx.http.get_json_value(&url).await {
                let mut versions = Vec::new();
                if let Some(releases) = resp.get("releases").and_then(|v| v.as_object()) {
                    for (ver, files) in releases {
                        // Skip versions with no files (yanked)
                        if files.as_array().map(|a| a.is_empty()).unwrap_or(true) {
                            continue;
                        }
                        let prerelease = ver.contains('a')
                            || ver.contains('b')
                            || ver.contains("rc")
                            || ver.contains("dev");
                        versions.push(VersionInfo {
                            version: ver.clone(),
                            released_at: None,
                            prerelease,
                            lts: false,
                            download_url: None,
                            checksum: None,
                            metadata: HashMap::new(),
                        });
                    }
                }
                // Sort newest first
                versions.sort_by(|a, b| {
                    let parse = |v: &str| -> Vec<u64> {
                        v.split(|c: char| !c.is_ascii_digit())
                            .filter(|s| !s.is_empty())
                            .filter_map(|s| s.parse::<u64>().ok())
                            .collect()
                    };
                    parse(&b.version).cmp(&parse(&a.version))
                });
                return Ok(versions);
            }
        }
        Ok(vec![VersionInfo {
            version: "system".to_string(),
            released_at: None,
            prerelease: false,
            lts: true,
            download_url: None,
            checksum: None,
            metadata: HashMap::new(),
        }])
    }

    /// Check if the tool is installed on the system
    async fn is_installed(&self, _version: &str, _ctx: &RuntimeContext) -> Result<bool> {
        Ok(which::which(&self.executable).is_ok())
    }

    /// Get installed versions
    async fn installed_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<String>> {
        if which::which(&self.executable).is_ok() {
            // Try to detect the version
            if let Ok(Some(version)) = self.detect_version().await {
                return Ok(vec![version]);
            }
            Ok(vec!["system".to_string()])
        } else {
            Ok(vec![])
        }
    }

    /// Get download URL (if direct download is available)
    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Prefer Starlark-driven download_url if injected
        if let Some(ref f) = self.download_url_fn {
            return f(version.to_string()).await;
        }

        // Fall back to scanning install_strategies for DirectDownload
        for strategy in &self.install_strategies {
            if let InstallStrategy::DirectDownload { url, platforms, .. } = strategy
                && (platforms.is_empty()
                    || platforms
                        .iter()
                        .any(|p| p.eq_ignore_ascii_case(platform.os_name())))
            {
                // Substitute version in URL
                let url = url.replace("{version}", version);
                return Ok(Some(url));
            }
        }
        Ok(None)
    }

    /// Install the runtime using the best available strategy
    ///
    /// This method tries installation strategies in priority order:
    /// 1. Starlark-driven install_layout (if injected) — uses custom strip_prefix/exe paths
    /// 2. Direct download URL (from download_url_fn or install_strategies)
    /// 3. System package managers (brew, choco, apt, etc.)
    /// 4. Installation scripts
    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let platform = Platform::current();
        let store_name = self.bundled_with.as_deref().unwrap_or(&self.name);
        let base_path = ctx.paths.version_store_dir(store_name, version);
        let install_path = base_path.join(platform.as_str());

        // pip package: install via uv pip install <package>==<version>
        if let Some(ref pkg) = self.pip_package {
            return crate::package_runtime::install_pip_package_for_manifest(
                pkg, &self.name, version, ctx,
            )
            .await;
        }

        // Try Starlark-driven install_layout first (provides URL + strip_prefix + exe paths)
        if let Some(ref layout_fn) = self.install_layout_fn
            && let Some(layout) = layout_fn(version.to_string()).await?
        {
            let url = layout
                .get("url")
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());

            if let Some(url) = url {
                info!(
                    "Installing {} via Starlark install_layout from {}",
                    self.name, url
                );

                if ctx.fs.exists(&install_path) {
                    let exe_path = self.resolve_exe_path_from_layout(&install_path, &layout);
                    return Ok(InstallResult::already_installed(
                        install_path,
                        exe_path,
                        version.to_string(),
                    ));
                }

                // Build layout metadata for download_with_layout
                let mut layout_meta = std::collections::HashMap::new();
                if let Some(prefix) = layout.get("strip_prefix").and_then(|s| s.as_str()) {
                    layout_meta.insert("strip_prefix".to_string(), prefix.to_string());
                }

                ctx.installer
                    .download_with_layout(&url, &install_path, &layout_meta)
                    .await?;

                let exe_path = self.resolve_exe_path_from_layout(&install_path, &layout);
                return Ok(InstallResult::success(
                    install_path,
                    exe_path,
                    version.to_string(),
                ));
            }
        }

        // First try the default install (direct download) if URL is available
        if let Some(url) = self.download_url(version, &platform).await? {
            info!("Installing {} via direct download from {}", self.name, url);

            // Resolve install_layout for executable path hints (strip_prefix, executable_paths)
            let layout_hint = if let Some(ref layout_fn) = self.install_layout_fn {
                match layout_fn(version.to_string()).await {
                    Ok(Some(layout)) => {
                        debug!("install_layout_fn returned: {:?}", layout);
                        Some(layout)
                    }
                    Ok(None) => {
                        debug!("install_layout_fn returned None");
                        None
                    }
                    Err(e) => {
                        warn!("install_layout_fn failed: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            if ctx.fs.exists(&install_path) {
                // Already installed — use layout hint to find the executable
                let exe_path = if let Some(ref layout) = layout_hint {
                    self.resolve_exe_path_from_layout(&install_path, layout)
                } else {
                    install_path.join(vx_paths::with_executable_extension(&self.executable))
                };
                return Ok(InstallResult::already_installed(
                    install_path,
                    exe_path,
                    version.to_string(),
                ));
            }

            // Build layout metadata for download_with_layout
            let mut layout_meta = std::collections::HashMap::new();
            if let Some(ref layout) = layout_hint {
                // Archive strip_prefix
                if let Some(prefix) = layout.get("strip_prefix").and_then(|s| s.as_str()) {
                    debug!("Using strip_prefix: {}", prefix);
                    layout_meta.insert("strip_prefix".to_string(), prefix.to_string());
                }
                // Binary rename: source_name → target_name in target_dir
                if let Some(source) = layout.get("source_name").and_then(|s| s.as_str()) {
                    layout_meta.insert("source_name".to_string(), source.to_string());
                }
                if let Some(target) = layout.get("target_name").and_then(|s| s.as_str()) {
                    layout_meta.insert("target_name".to_string(), target.to_string());
                }
                if let Some(dir) = layout.get("target_dir").and_then(|s| s.as_str()) {
                    layout_meta.insert("target_dir".to_string(), dir.to_string());
                }
            }
            debug!("layout_meta for download_with_layout: {:?}", layout_meta);

            ctx.installer
                .download_with_layout(&url, &install_path, &layout_meta)
                .await?;

            let exe_path = if let Some(ref layout) = layout_hint {
                self.resolve_exe_path_from_layout(&install_path, layout)
            } else {
                install_path.join(vx_paths::with_executable_extension(&self.executable))
            };
            return Ok(InstallResult::success(
                install_path,
                exe_path,
                version.to_string(),
            ));
        }

        // No direct download, try system package manager strategies
        info!(
            "No direct download for {} on {:?}, trying system package managers",
            self.name, platform.os
        );

        // Get applicable strategies sorted by priority
        let mut strategies: Vec<_> = self
            .install_strategies
            .iter()
            .filter(|s| s.matches_platform(&platform))
            .collect();
        strategies.sort_by_key(|s| std::cmp::Reverse(s.priority()));

        let registry = PackageManagerRegistry::new();
        let available_managers = registry.get_available().await;

        for strategy in strategies {
            match strategy {
                InstallStrategy::PackageManager {
                    manager,
                    package,
                    params,
                    install_args,
                    ..
                } => {
                    // Check if this package manager is available
                    let pm = available_managers
                        .iter()
                        .find(|pm| pm.name().eq_ignore_ascii_case(manager));

                    if let Some(pm) = pm {
                        debug!(
                            "Trying to install {} via {} (package: {})",
                            self.name, manager, package
                        );

                        let spec = PackageInstallSpec {
                            package: package.clone(),
                            params: params.clone(),
                            install_args: install_args.clone(),
                            ..Default::default()
                        };

                        match pm.install_package(&spec).await {
                            Ok(_) => {
                                info!("Successfully installed {} via {}", self.name, manager);
                                // System-installed, find the executable path
                                let exe_path = which::which(&self.executable).ok();
                                return Ok(InstallResult::system_installed(
                                    format!("system ({})", manager),
                                    exe_path,
                                ));
                            }
                            Err(e) => {
                                warn!("Failed to install {} via {}: {}", self.name, manager, e);
                                continue;
                            }
                        }
                    } else {
                        debug!("Package manager {} not available, skipping", manager);
                    }
                }
                InstallStrategy::Script {
                    url,
                    script_type,
                    args,
                    ..
                } => {
                    debug!("Script installation not yet implemented for {}", self.name);
                    // TODO: Implement script-based installation
                    let _ = (url, script_type, args);
                }
                InstallStrategy::ProvidedBy {
                    provider,
                    relative_path,
                    ..
                } => {
                    // Check if the provider runtime is installed
                    if which::which(provider).is_ok() {
                        debug!("{} is provided by {}", self.name, provider);
                        let exe_path = PathBuf::from(relative_path);
                        return Ok(InstallResult::system_installed(
                            format!("provided by {}", provider),
                            Some(exe_path),
                        ));
                    }
                }
                InstallStrategy::DirectDownload { .. } => {
                    // Already tried above
                }
            }
        }

        // All strategies failed
        let tried_managers: Vec<_> = self
            .install_strategies
            .iter()
            .filter_map(|s| match s {
                InstallStrategy::PackageManager { manager, .. } => Some(manager.as_str()),
                _ => None,
            })
            .collect();

        if tried_managers.is_empty() {
            Err(anyhow::anyhow!(
                "No installation strategy available for {} on this platform",
                self.name
            ))
        } else {
            Err(anyhow::anyhow!(
                "Failed to install {}. Tried package managers: {}.\n\
                 Please ensure a package manager is installed (brew, choco, scoop, apt, etc.) \
                 and try again.",
                self.name,
                tried_managers.join(", ")
            ))
        }
    }

    fn normalize_config(&self) -> Option<&NormalizeConfig> {
        self.normalize.as_ref()
    }

    // ========== Shell Support (RFC 0038) ==========

    fn get_shell_path(
        &self,
        shell_name: &str,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Option<std::path::PathBuf> {
        // Find the shell definition
        let shell_def = self.shells.iter().find(|s| s.name == shell_name)?;
        let shell_relative = &shell_def.path;

        tracing::debug!(
            "Looking for shell '{}' with relative path '{}'",
            shell_name,
            shell_relative
        );

        // 1. Try vx store directory first
        let platform = Platform::current();
        let store_name = self.store_name();
        let base_path = ctx.paths.version_store_dir(store_name, version);
        let install_path = base_path.join(platform.as_str());
        let shell_path = install_path.join(shell_relative);

        tracing::debug!("Checking vx store path: {}", shell_path.display());

        if shell_path.exists() {
            tracing::debug!("Found shell in vx store: {}", shell_path.display());
            return Some(shell_path);
        }

        // 2. Try system paths from detection config
        if let Some(ref detection) = self.detection {
            for sys_path in &detection.system_paths {
                let path = std::path::PathBuf::from(sys_path);
                if path.exists() {
                    // Found the executable, derive install directory
                    if let Some(install_dir) = derive_install_dir_from_executable(&path) {
                        let shell_in_install = install_dir.join(shell_relative);
                        tracing::debug!(
                            "Checking system install path: {}",
                            shell_in_install.display()
                        );
                        if shell_in_install.exists() {
                            tracing::debug!(
                                "Found shell in system install: {}",
                                shell_in_install.display()
                            );
                            return Some(shell_in_install);
                        }
                    }
                }
            }
        }

        // 3. Try to find executable in PATH and derive install directory
        if let Ok(exe_path) = which::which(&self.executable) {
            tracing::debug!(
                "Found executable '{}' at: {}",
                self.executable,
                exe_path.display()
            );
            if let Some(install_dir) = derive_install_dir_from_executable(&exe_path) {
                let shell_in_install = install_dir.join(shell_relative);
                tracing::debug!(
                    "Checking derived install path: {}",
                    shell_in_install.display()
                );
                if shell_in_install.exists() {
                    tracing::debug!(
                        "Found shell in derived install: {}",
                        shell_in_install.display()
                    );
                    return Some(shell_in_install);
                }
            }
        }

        // 4. Try to find shell directly in PATH (last resort)
        if let Ok(shell_exe) = which::which(shell_name) {
            tracing::debug!("Found shell in PATH: {}", shell_exe.display());
            return Some(shell_exe);
        }

        tracing::debug!("Shell '{}' not found", shell_name);
        None
    }

    fn provided_shells(&self) -> Vec<&'static str> {
        // This is a limitation - we can't return borrowed slices from owned Vec
        // Return empty for now, the get_shell_path method will still work
        vec![]
    }
}

/// Check if a package manager is available on the system
async fn is_package_manager_available(manager: &str) -> bool {
    match manager {
        "choco" | "chocolatey" => which::which("choco").is_ok(),
        "winget" => which::which("winget").is_ok(),
        "scoop" => which::which("scoop").is_ok(),
        "brew" | "homebrew" => which::which("brew").is_ok(),
        "apt" | "apt-get" => which::which("apt").is_ok() || which::which("apt-get").is_ok(),
        "yum" => which::which("yum").is_ok(),
        "dnf" => which::which("dnf").is_ok(),
        "pacman" => which::which("pacman").is_ok(),
        "zypper" => which::which("zypper").is_ok(),
        "apk" => which::which("apk").is_ok(),
        _ => false,
    }
}

/// Derive the installation directory from an executable path.
///
/// This is used to find the root installation directory for system-installed tools.
/// For example, Git for Windows installs git.exe to:
/// - `C:\Program Files\Git\cmd\git.exe` or `C:\Program Files\Git\bin\git.exe`
///
/// The install directory would be `C:\Program Files\Git`.
fn derive_install_dir_from_executable(exe_path: &std::path::Path) -> Option<std::path::PathBuf> {
    // Get the parent directory of the executable
    let parent = exe_path.parent()?;

    // Common patterns for installation directories:
    // - Windows: <install>/bin/..., <install>/cmd/..., <install>/...
    // - Unix: <install>/bin/...

    let parent_name = parent.file_name()?.to_str()?;

    // Check if parent is a common bin directory
    if matches!(parent_name, "bin" | "cmd" | "sbin" | "libexec" | "Scripts") {
        // The install directory is the parent of the bin directory
        return parent.parent().map(|p| p.to_path_buf());
    }

    // On Windows, also check for mingw64/bin pattern (Git for Windows)
    #[cfg(windows)]
    {
        if parent_name == "bin"
            && let Some(grandparent) = parent.parent()
            && let Some(grandparent_name) = grandparent.file_name().and_then(|n| n.to_str())
            && (grandparent_name == "mingw64" || grandparent_name == "mingw32")
        {
            // Git for Windows: <install>/mingw64/bin/
            return grandparent.parent().map(|p| p.to_path_buf());
        }
    }

    // Otherwise, assume the parent is the install directory
    Some(parent.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_strategy_priority() {
        let strategy = InstallStrategy::PackageManager {
            manager: "choco".to_string(),
            package: "git".to_string(),
            params: None,
            install_args: None,
            priority: 80,
            platforms: vec!["windows".to_string()],
        };

        assert_eq!(strategy.priority(), 80);
    }

    #[test]
    fn test_install_strategy_platform_filter() {
        let strategy = InstallStrategy::PackageManager {
            manager: "brew".to_string(),
            package: "git".to_string(),
            params: None,
            install_args: None,
            priority: 90,
            platforms: vec!["macos".to_string(), "linux".to_string()],
        };

        let macos = Platform::new(crate::Os::MacOS, crate::Arch::Aarch64);
        let windows = Platform::new(crate::Os::Windows, crate::Arch::X86_64);

        assert!(strategy.matches_platform(&macos));
        assert!(!strategy.matches_platform(&windows));
    }

    #[test]
    fn test_manifest_runtime_builder() {
        let runtime = ManifestDrivenRuntime::new("fd", "mytools", ProviderSource::BuiltIn)
            .with_description("A simple, fast alternative to find")
            .with_executable("fd")
            .with_alias("fd-find")
            .with_strategy(InstallStrategy::PackageManager {
                manager: "brew".to_string(),
                package: "fd".to_string(),
                params: None,
                install_args: None,
                priority: 90,
                platforms: vec![],
            });

        assert_eq!(runtime.name(), "fd");
        assert_eq!(runtime.description(), "A simple, fast alternative to find");
        assert_eq!(runtime.install_strategies.len(), 1);
    }
}
