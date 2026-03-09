//! Runtime resolver - detects and resolves runtime dependencies
//!
//! This module handles:
//! - Detecting installed runtimes (both vx-managed and system)
//! - Resolving runtime dependencies
//! - Determining what needs to be installed
//! - Checking dependency version constraints

use crate::{ResolverConfig, Result, RuntimeDependency, RuntimeMap, RuntimeSpec};
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::{trace, warn};
use vx_paths::PathResolver as VxPathResolver;

/// Matches full semver: 1.2.3 or v1.2.3
static VERSION_REGEX: OnceLock<Regex> = OnceLock::new();
/// Matches major.minor: 1.2 or v1.2
static VERSION_REGEX_SIMPLE: OnceLock<Regex> = OnceLock::new();

fn version_regex() -> &'static Regex {
    VERSION_REGEX.get_or_init(|| Regex::new(r"v?(\d+\.\d+\.\d+)").expect("valid regex"))
}

fn version_regex_simple() -> &'static Regex {
    VERSION_REGEX_SIMPLE.get_or_init(|| Regex::new(r"v?(\d+\.\d+)").expect("valid regex"))
}

/// Status of a runtime
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeStatus {
    /// Runtime is managed by vx and installed
    VxManaged { version: String, path: PathBuf },
    /// Runtime is available in system PATH
    SystemAvailable { path: PathBuf },
    /// Runtime is not installed
    NotInstalled,
    /// Runtime is unknown (not in runtime map)
    Unknown,
}

impl RuntimeStatus {
    /// Check if the runtime is available (either vx-managed or system)
    pub fn is_available(&self) -> bool {
        matches!(
            self,
            RuntimeStatus::VxManaged { .. } | RuntimeStatus::SystemAvailable { .. }
        )
    }

    /// Get the executable path if available
    pub fn executable_path(&self) -> Option<&PathBuf> {
        match self {
            RuntimeStatus::VxManaged { path, .. } => Some(path),
            RuntimeStatus::SystemAvailable { path } => Some(path),
            _ => None,
        }
    }
}

/// Information about a dependency that doesn't meet version constraints
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IncompatibleDependency {
    /// Name of the dependency runtime
    pub runtime_name: String,
    /// Current installed version (if any)
    pub current_version: Option<String>,
    /// The dependency constraint
    pub constraint: RuntimeDependency,
    /// Recommended version to use/install
    pub recommended_version: Option<String>,
}

/// Information about a runtime that is not supported on the current platform
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnsupportedPlatformRuntime {
    /// Name of the runtime
    pub runtime_name: String,
    /// Current platform
    pub current_platform: String,
    /// Supported platforms (human-readable)
    pub supported_platforms: String,
    /// Whether this is the primary runtime or a dependency
    pub is_primary: bool,
}

/// Resolution result for a runtime execution request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResolutionResult {
    /// The primary runtime to execute
    pub runtime: String,

    /// The actual executable to run
    pub executable: PathBuf,

    /// Command prefix to add before user arguments
    pub command_prefix: Vec<String>,

    /// Runtimes that need to be installed before execution
    pub missing_dependencies: Vec<String>,

    /// Installation order for missing dependencies
    pub install_order: Vec<String>,

    /// Whether the runtime itself needs installation
    pub runtime_needs_install: bool,

    /// Dependencies that are installed but don't meet version constraints
    pub incompatible_dependencies: Vec<IncompatibleDependency>,

    /// Full dependency requirements collected during resolution.
    #[serde(default)]
    pub dependency_requirements: Vec<RuntimeDependency>,

    /// Runtimes that are not supported on the current platform
    pub unsupported_platform_runtimes: Vec<UnsupportedPlatformRuntime>,
}

/// Runtime resolver that handles dependency detection and resolution
pub struct Resolver {
    /// Runtime map for runtime specifications
    runtime_map: RuntimeMap,

    /// Path resolver for vx-managed runtimes
    path_resolver: VxPathResolver,

    /// Configuration
    config: ResolverConfig,
}

impl Resolver {
    /// Create a resolver with a runtime map
    ///
    /// The RuntimeMap should be built from provider manifests using
    /// `RuntimeMap::from_manifests()`. See RFC 0017.
    pub fn new(config: ResolverConfig, runtime_map: RuntimeMap) -> Result<Self> {
        let path_resolver = VxPathResolver::default_paths()?;
        Ok(Self {
            runtime_map,
            path_resolver,
            config,
        })
    }

    /// Persist the exec path cache to disk.
    ///
    /// Should be called after the pipeline completes to save any new
    /// cache entries discovered during executable resolution.
    pub fn save_exec_cache(&self) {
        self.path_resolver.save_cache();
    }

    /// Check the status of a runtime.
    ///
    /// Probes locations in a fixed priority order determined by config:
    /// 1. vx-managed store  (when `prefer_vx_managed`)
    /// 2. system PATH        (when `fallback_to_system`)
    /// 3. vx-managed store  (when *not* `prefer_vx_managed`)
    /// 4. detection paths    (glob patterns from provider.toml, e.g. MSVC)
    pub fn check_runtime_status(&self, runtime_name: &str) -> RuntimeStatus {
        let spec = self.runtime_map.get(runtime_name);
        let resolved_name = spec.map(|s| s.name.as_str()).unwrap_or(runtime_name);
        let executable_name = spec.map(|s| s.get_executable()).unwrap_or(runtime_name);
        let store_dir_name = self.get_store_directory_name(spec, resolved_name);

        // vx-managed (high priority)
        if self.config.prefer_vx_managed
            && let Some(status) = self.check_vx_managed(store_dir_name, executable_name)
        {
            return status;
        }

        // system PATH
        if self.config.fallback_to_system {
            let status = self.check_system_path(executable_name);
            if status.is_available() {
                return status;
            }
        }

        // vx-managed (low priority)
        if !self.config.prefer_vx_managed
            && let Some(status) = self.check_vx_managed(store_dir_name, executable_name)
        {
            return status;
        }

        // detection glob paths (last resort)
        if let Some(status) = self.check_detection_paths(runtime_name, executable_name) {
            return status;
        }

        RuntimeStatus::NotInstalled
    }

    /// Get the store directory name for a runtime
    /// For bundled runtimes, this returns the parent runtime's name
    fn get_store_directory_name<'a>(
        &self,
        spec: Option<&'a RuntimeSpec>,
        default_name: &'a str,
    ) -> &'a str {
        if let Some(spec) = spec {
            // Check if this runtime is bundled with another runtime
            for dep in &spec.dependencies {
                if let Some(ref provided_by) = dep.provided_by {
                    // This is a bundled runtime, use the parent's directory
                    return provided_by.as_str();
                }
            }
        }
        default_name
    }

    /// Check if a runtime is installed via vx
    fn check_vx_managed(&self, runtime_name: &str, executable_name: &str) -> Option<RuntimeStatus> {
        // Use unified path resolver to find the tool
        match self
            .path_resolver
            .find_tool_with_executable(runtime_name, executable_name)
        {
            Ok(Some(location)) => {
                trace!(
                    "found {} {} at {}",
                    runtime_name,
                    location.version,
                    location.path.display()
                );
                Some(RuntimeStatus::VxManaged {
                    version: location.version,
                    path: location.path,
                })
            }
            Ok(None) => {
                trace!("tool {} not in store", runtime_name);
                None
            }
            Err(e) => {
                trace!("error checking {}: {}", runtime_name, e);
                None
            }
        }
    }

    /// Check if a runtime is available in system PATH
    fn check_system_path(&self, runtime_name: &str) -> RuntimeStatus {
        match which::which(runtime_name) {
            Ok(path) => {
                trace!("Found {} in system PATH: {:?}", runtime_name, path);
                RuntimeStatus::SystemAvailable { path }
            }
            Err(_) => RuntimeStatus::NotInstalled,
        }
    }

    /// Check detection system_paths (glob patterns) for a runtime executable
    ///
    /// This handles runtimes like MSVC where the executable (cl.exe) is not in
    /// the system PATH but can be found via known installation paths defined
    /// in provider.toml detection config.
    fn check_detection_paths(
        &self,
        runtime_name: &str,
        executable_name: &str,
    ) -> Option<RuntimeStatus> {
        // When executable_name differs from runtime_name, we are in the
        // `runtime::executable` override scenario (e.g. `vx msvc::ildasm`).
        //
        // Priority:
        //   1. If `executable_name` is itself a known runtime with its own
        //      system_paths (e.g. `ildasm`), use those paths.
        //   2. If `executable_name` equals `runtime_name` (normal case), use
        //      the runtime's own system_paths.
        //   3. Otherwise the executable is unknown — return None so the caller
        //      can report a proper "not found" error instead of silently
        //      resolving to the parent runtime's binary (e.g. cl.exe).
        let system_paths = if executable_name == runtime_name {
            // Normal case: look up the runtime's own detection paths.
            self.runtime_map.get_detection_system_paths(runtime_name)
        } else if self.runtime_map.get(executable_name).is_some() {
            // executable_name is a known runtime — prefer its own system_paths.
            let exe_paths = self.runtime_map.get_detection_system_paths(executable_name);
            if !exe_paths.is_empty() {
                exe_paths
            } else {
                // Known runtime but no dedicated system_paths — fall back to
                // the parent runtime's paths (bundled tool in the same directory).
                self.runtime_map.get_detection_system_paths(runtime_name)
            }
        } else {
            // executable_name is NOT a known runtime.  Do NOT fall back to the
            // parent runtime's system_paths — that would silently resolve an
            // unknown tool (e.g. `sigtools`) to the parent's binary (cl.exe).
            return None;
        };

        if system_paths.is_empty() {
            return None;
        }

        trace!(
            "Checking detection system_paths for {} (exe: {}): {} patterns",
            runtime_name,
            executable_name,
            system_paths.len()
        );

        for pattern in &system_paths {
            match glob::glob(pattern) {
                Ok(paths) => {
                    // Collect and sort matches to get the latest version (typically last alphabetically)
                    let mut matches: Vec<PathBuf> = paths
                        .filter_map(|p| p.ok())
                        .filter(|p| p.exists())
                        .collect();
                    // Sort descending so the latest version comes first
                    matches.sort_by(|a, b| b.cmp(a));

                    if let Some(path) = matches.into_iter().next() {
                        trace!(
                            "Found {} via detection system_paths: {}",
                            runtime_name,
                            path.display()
                        );
                        return Some(RuntimeStatus::SystemAvailable { path });
                    }
                }
                Err(e) => {
                    trace!("Invalid glob pattern '{}': {}", pattern, e);
                }
            }
        }

        None
    }

    /// Resolve a runtime for execution
    ///
    /// This method:
    /// 1. Looks up the runtime specification
    /// 2. Checks if the runtime and its dependencies are installed
    /// 3. Returns a resolution result with execution details
    pub fn resolve(&self, runtime_name: &str) -> Result<ResolutionResult> {
        self.resolve_with_version(runtime_name, None)
    }

    /// Resolve a runtime for execution with a specific version
    ///
    /// This method:
    /// 1. Looks up the runtime specification
    /// 2. Checks if the specific version is installed
    /// 3. Checks dependency version constraints
    /// 4. Returns a resolution result with execution details
    pub fn resolve_with_version(
        &self,
        runtime_name: &str,
        version: Option<&str>,
    ) -> Result<ResolutionResult> {
        // Get the runtime specification
        let spec = self.runtime_map.get(runtime_name);
        trace!(
            "spec: {} has {} dependencies",
            runtime_name,
            spec.map(|s| s.dependencies.len()).unwrap_or(0)
        );

        // Check platform compatibility first
        let unsupported_platform_runtimes = Vec::new();

        // Note: Platform compatibility checking is done at the CLI layer
        // where we have access to the ProviderRegistry. The resolver
        // only handles dependency resolution.

        // Check runtime status (optionally with specific version)
        let runtime_status = if let Some(ver) = version {
            self.check_runtime_status_with_version(runtime_name, ver)
        } else {
            self.check_runtime_status(runtime_name)
        };

        // Collect missing dependencies and incompatible dependencies
        let mut missing_deps = Vec::new();
        let mut install_order = Vec::new();
        let mut incompatible_deps = Vec::new();
        let mut dependency_requirements = Vec::new();

        if let Some(spec) = spec {
            // Check each required dependency
            for dep in spec.required_dependencies() {
                dependency_requirements.push(dep.clone());

                let dep_name = dep.provided_by.as_deref().unwrap_or(&dep.runtime_name);

                // Note: Platform compatibility checking for dependencies is done at the CLI layer

                let dep_status = self.check_runtime_status(dep_name);

                trace!(
                    "checking dep {} (min: {:?}, max: {:?})",
                    dep_name, dep.min_version, dep.max_version
                );

                match &dep_status {
                    RuntimeStatus::VxManaged { version, .. } => {
                        // Check version constraints
                        let is_compatible = dep.is_version_compatible(version);
                        trace!(
                            "dep {} {} is {}",
                            dep_name,
                            version,
                            if is_compatible { "ok" } else { "INCOMPATIBLE" }
                        );
                        if !is_compatible {
                            warn!(
                                "Dependency {} version {} does not meet constraints for {} (min: {:?}, max: {:?})",
                                dep_name, version, runtime_name, dep.min_version, dep.max_version
                            );
                            incompatible_deps.push(IncompatibleDependency {
                                runtime_name: dep_name.to_string(),
                                current_version: Some(version.clone()),
                                constraint: dep.clone(),
                                recommended_version: dep.recommended_version.clone(),
                            });
                        }
                    }
                    RuntimeStatus::SystemAvailable { path } => {
                        // Try to get version from system runtime
                        if let Some(system_version) =
                            self.get_system_runtime_version(dep_name, path)
                            && !dep.is_version_compatible(&system_version)
                        {
                            warn!(
                                "System {} version {} does not meet constraints for {} (min: {:?}, max: {:?})",
                                dep_name,
                                system_version,
                                runtime_name,
                                dep.min_version,
                                dep.max_version
                            );
                            incompatible_deps.push(IncompatibleDependency {
                                runtime_name: dep_name.to_string(),
                                current_version: Some(system_version),
                                constraint: dep.clone(),
                                recommended_version: dep.recommended_version.clone(),
                            });
                        }
                    }
                    RuntimeStatus::NotInstalled | RuntimeStatus::Unknown => {
                        trace!("missing dep: {}", dep_name);
                        missing_deps.push(dep_name.to_string());
                    }
                }
            }

            // Get installation order
            if !missing_deps.is_empty() {
                install_order = self.get_install_order(&missing_deps);
            }
        }

        // Determine if runtime itself needs installation
        let runtime_needs_install = !runtime_status.is_available();
        if runtime_needs_install {
            // Add runtime to install order if not already there
            let resolved_name = self
                .runtime_map
                .resolve_name(runtime_name)
                .unwrap_or(runtime_name);
            if !install_order.contains(&resolved_name.to_string()) {
                install_order.push(resolved_name.to_string());
            }
        }

        // Get executable path
        let executable = match &runtime_status {
            RuntimeStatus::VxManaged { path, .. } => path.clone(),
            RuntimeStatus::SystemAvailable { path } => path.clone(),
            _ => {
                // Runtime not installed - use the expected executable name
                let exe_name = spec.map(|s| s.get_executable()).unwrap_or(runtime_name);
                PathBuf::from(exe_name)
            }
        };

        // Get command prefix if applicable
        let command_prefix = spec.map(|s| s.command_prefix.clone()).unwrap_or_default();

        Ok(ResolutionResult {
            runtime: runtime_name.to_string(),
            executable,
            command_prefix,
            missing_dependencies: missing_deps,
            install_order,
            runtime_needs_install,
            incompatible_dependencies: incompatible_deps,
            dependency_requirements,
            unsupported_platform_runtimes,
        })
    }

    /// Resolve a runtime with an executable override.
    ///
    /// This supports the `runtime::executable` syntax (e.g., `vx msvc::cl`).
    /// The runtime name is used for store directory lookup, dependency resolution,
    /// and installation, while the executable name is used for finding the actual
    /// binary to execute.
    ///
    /// Special case: When the executable is a known shell (cmd, powershell, pwsh,
    /// bash, zsh, sh), we find it in system PATH but still apply the runtime's
    /// environment variables. This allows commands like `vx msvc::cmd` to spawn
    /// a shell with MSVC environment.
    pub fn resolve_with_executable(
        &self,
        runtime_name: &str,
        version: Option<&str>,
        executable_name: &str,
    ) -> Result<ResolutionResult> {
        // First resolve normally to get dependencies etc.
        let mut result = self.resolve_with_version(runtime_name, version)?;

        // Special handling for shell executables
        // When user runs `vx msvc::cmd`, they want a shell with MSVC environment
        if Self::is_shell_executable(executable_name) {
            // Find shell in system PATH
            let shell_exe = if cfg!(windows) {
                format!("{}.exe", executable_name)
            } else {
                executable_name.to_string()
            };
            if let Ok(path) = which::which(&shell_exe) {
                trace!(
                    "Found shell '{}' in system PATH for runtime '{}': {}",
                    executable_name,
                    runtime_name,
                    path.display()
                );
                result.executable = path;
                result.runtime_needs_install = false;
                return Ok(result);
            }
            // Shell not found, fall through to normal resolution
        }

        // Re-resolve the executable path using the override name.
        // This applies both when the runtime is already available AND when it was
        // found via detection paths (e.g., MSVC cl.exe in Visual Studio directories).
        let spec = self.runtime_map.get(runtime_name);
        let store_dir_name = self.get_store_directory_name(spec, runtime_name);

        // Probe locations in priority order and return on first match.
        let candidates: [(&str, Option<PathBuf>); 4] = [
            // 1. vx-managed store (when preferred)
            (
                "vx-store (preferred)",
                if self.config.prefer_vx_managed {
                    self.check_vx_managed(store_dir_name, executable_name)
                        .and_then(|s| s.executable_path().cloned())
                } else {
                    None
                },
            ),
            // 2. system PATH
            (
                "system PATH",
                if self.config.fallback_to_system {
                    self.check_system_path(executable_name)
                        .executable_path()
                        .cloned()
                } else {
                    None
                },
            ),
            // 3. vx-managed store (fallback when not preferred)
            (
                "vx-store (fallback)",
                if !self.config.prefer_vx_managed {
                    self.check_vx_managed(store_dir_name, executable_name)
                        .and_then(|s| s.executable_path().cloned())
                } else {
                    None
                },
            ),
            // 4. detection system_paths (glob patterns from provider.toml, e.g. MSVC)
            (
                "detection paths",
                self.check_detection_paths(runtime_name, executable_name)
                    .and_then(|s| s.executable_path().cloned()),
            ),
        ];

        for (label, found) in candidates {
            if let Some(path) = found {
                trace!(
                    "Resolved '{}' via {}: {}",
                    executable_name,
                    label,
                    path.display()
                );
                result.executable = path;
                result.runtime_needs_install = false;
                return Ok(result);
            }
        }

        // Executable override not found — set bare name as fallback
        // (PrepareStage will report the appropriate error)
        if !result.runtime_needs_install {
            result.executable = PathBuf::from(executable_name);
        }

        Ok(result)
    }

    /// Check if an executable name is a known shell
    fn is_shell_executable(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        // Common shells - check if the name matches any known shell
        let common_shells = ["cmd", "powershell", "pwsh", "bash", "sh", "zsh", "fish"];

        if common_shells.contains(&name_lower.as_str()) {
            return true;
        }
        // Unix-specific shells
        #[cfg(not(windows))]
        {
            let unix_shells = ["dash", "ksh", "csh", "tcsh"];
            if unix_shells.contains(&name_lower.as_str()) {
                return true;
            }
        }
        false
    }

    /// Get the version of a system runtime by running `<runtime> --version`
    fn get_system_runtime_version(&self, runtime_name: &str, path: &PathBuf) -> Option<String> {
        use std::process::Command;

        let output = Command::new(path).arg("--version").output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        // Extract version number from output
        // Common patterns: "v18.17.0", "18.17.0", "node v18.17.0"
        self.extract_version_from_output(&combined, runtime_name)
    }

    /// Extract version number from command output
    fn extract_version_from_output(&self, output: &str, _runtime_name: &str) -> Option<String> {
        // Try full semver pattern first: 1.2.3 or v1.2.3
        if let Some(captures) = version_regex().captures(output) {
            return Some(captures.get(1)?.as_str().to_string());
        }

        // Fall back to major.minor only
        if let Some(captures) = version_regex_simple().captures(output) {
            return Some(captures.get(1)?.as_str().to_string());
        }

        None
    }

    /// Check the status of a runtime with a specific version
    pub fn check_runtime_status_with_version(
        &self,
        runtime_name: &str,
        version: &str,
    ) -> RuntimeStatus {
        // Get the runtime specification if known
        let spec = self.runtime_map.get(runtime_name);
        let resolved_name = spec.map(|s| s.name.as_str()).unwrap_or(runtime_name);
        let executable_name = spec.map(|s| s.get_executable()).unwrap_or(runtime_name);

        // For bundled runtimes, use the parent runtime's directory
        let store_dir_name = self.get_store_directory_name(spec, resolved_name);

        // Check vx-managed installation for specific version
        if let Some(status) =
            self.check_vx_managed_version(store_dir_name, executable_name, version)
        {
            return status;
        }

        RuntimeStatus::NotInstalled
    }

    /// Check if a specific version of a runtime is installed via vx
    fn check_vx_managed_version(
        &self,
        runtime_name: &str,
        executable_name: &str,
        version: &str,
    ) -> Option<RuntimeStatus> {
        // Use unified path resolver to find the specific version
        match self.path_resolver.find_tool_version_with_executable(
            runtime_name,
            version,
            executable_name,
        ) {
            Some(location) => {
                trace!(
                    "found {} {} at {}",
                    runtime_name,
                    location.version,
                    location.path.display()
                );
                Some(RuntimeStatus::VxManaged {
                    version: location.version,
                    path: location.path,
                })
            }
            None => {
                trace!("{} {} not in store", runtime_name, version);
                None
            }
        }
    }

    /// Merge additional dependency requirements into an existing resolution.
    ///
    /// This is used for version-aware dependencies discovered outside the static
    /// `RuntimeMap`, such as provider.star `deps(ctx, version)` hooks.
    pub fn merge_additional_dependencies(
        &self,
        runtime_name: &str,
        resolution: &mut ResolutionResult,
        deps: impl IntoIterator<Item = RuntimeDependency>,
    ) {
        for dep in deps {
            let dep_name = dep
                .provided_by
                .as_deref()
                .unwrap_or(&dep.runtime_name)
                .to_string();

            if let Some(existing) = resolution
                .dependency_requirements
                .iter_mut()
                .find(|existing| {
                    existing.runtime_name == dep.runtime_name
                        && existing.provided_by == dep.provided_by
                })
            {
                if existing.min_version.is_none() {
                    existing.min_version = dep.min_version.clone();
                }
                if existing.max_version.is_none() {
                    existing.max_version = dep.max_version.clone();
                }
                if existing.recommended_version.is_none() {
                    existing.recommended_version = dep.recommended_version.clone();
                }
                if existing.reason.is_empty() && !dep.reason.is_empty() {
                    existing.reason = dep.reason.clone();
                }
                existing.required |= dep.required;
            } else {
                resolution.dependency_requirements.push(dep.clone());
            }

            if !dep.required {
                continue;
            }

            if resolution
                .incompatible_dependencies
                .iter()
                .any(|existing| existing.runtime_name == dep_name)
            {
                continue;
            }

            if resolution
                .missing_dependencies
                .iter()
                .any(|existing| existing == &dep_name)
            {
                continue;
            }

            match self.check_runtime_status(&dep_name) {
                RuntimeStatus::VxManaged { version, .. } => {
                    if !dep.is_version_compatible(&version) {
                        resolution
                            .incompatible_dependencies
                            .push(IncompatibleDependency {
                                runtime_name: dep_name,
                                current_version: Some(version),
                                constraint: dep,
                                recommended_version: None,
                            });
                    }
                }
                RuntimeStatus::SystemAvailable { path } => {
                    if let Some(system_version) = self.get_system_runtime_version(&dep_name, &path)
                        && !dep.is_version_compatible(&system_version)
                    {
                        resolution
                            .incompatible_dependencies
                            .push(IncompatibleDependency {
                                runtime_name: dep_name,
                                current_version: Some(system_version),
                                constraint: dep,
                                recommended_version: None,
                            });
                    }
                }
                RuntimeStatus::NotInstalled | RuntimeStatus::Unknown => {
                    resolution.missing_dependencies.push(dep_name);
                }
            }
        }

        resolution.missing_dependencies.sort();
        resolution.missing_dependencies.dedup();

        if !resolution.missing_dependencies.is_empty() {
            resolution.install_order = self.get_install_order(&resolution.missing_dependencies);
            if resolution.runtime_needs_install {
                let resolved_name = self
                    .runtime_map
                    .resolve_name(runtime_name)
                    .unwrap_or(runtime_name)
                    .to_string();
                if !resolution
                    .install_order
                    .iter()
                    .any(|name| name == &resolved_name)
                {
                    resolution.install_order.push(resolved_name);
                }
            }
        }
    }

    /// Find the executable path for an installed runtime version.
    ///
    /// This is used when we know a runtime is installed but need its executable path
    /// (e.g., after `is_installed()` returns true).
    pub fn find_executable(&self, runtime_name: &str, version: &str) -> Option<PathBuf> {
        let spec = self.runtime_map.get(runtime_name);
        let resolved_name = spec.map(|s| s.name.as_str()).unwrap_or(runtime_name);
        let executable_name = spec.map(|s| s.get_executable()).unwrap_or(runtime_name);
        let store_dir_name = self.get_store_directory_name(spec, resolved_name);

        let platform_store_dir = self
            .path_resolver
            .manager()
            .platform_store_dir(store_dir_name, version);
        self.path_resolver
            .find_executable_in_dir(&platform_store_dir, executable_name)
    }

    /// Get the installation order for a set of runtimes
    fn get_install_order(&self, runtimes: &[String]) -> Vec<String> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();

        for runtime in runtimes {
            let runtime_order = self.runtime_map.get_install_order(runtime);
            for t in runtime_order {
                if !visited.contains(t) {
                    visited.insert(t);
                    order.push(t.to_string());
                }
            }
        }

        order
    }

    /// Get the runtime specification
    pub fn get_spec(&self, runtime_name: &str) -> Option<&RuntimeSpec> {
        self.runtime_map.get(runtime_name)
    }

    /// Get version-specific dependencies for a runtime
    ///
    /// This method queries the original RuntimeDef constraints to find
    /// dependencies that apply to a specific version.
    pub fn get_dependencies_for_version(
        &self,
        runtime_name: &str,
        version: &str,
    ) -> Vec<RuntimeDependency> {
        self.runtime_map
            .get_dependencies_for_version(runtime_name, version)
    }

    /// Get the parent runtime (provided_by) for a specific version
    ///
    /// Returns the runtime that provides this runtime for the given version.
    /// For example, Yarn 2.x+ returns "node" because it's provided via corepack.
    pub fn get_parent_runtime_for_version(
        &self,
        runtime_name: &str,
        version: &str,
    ) -> Option<String> {
        self.runtime_map
            .get_parent_runtime_for_version(runtime_name, version)
    }

    /// Check if a runtime is known
    pub fn is_known_runtime(&self, runtime_name: &str) -> bool {
        self.runtime_map.contains(runtime_name)
    }

    /// Get all known runtime names
    pub fn known_runtimes(&self) -> Vec<&str> {
        self.runtime_map.runtime_names()
    }

    /// Get the configuration
    pub fn config(&self) -> &ResolverConfig {
        &self.config
    }
}
