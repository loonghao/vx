//! Runtime resolver - detects and resolves runtime dependencies
//!
//! This module handles:
//! - Detecting installed runtimes (both vx-managed and system)
//! - Resolving runtime dependencies
//! - Determining what needs to be installed
//! - Checking dependency version constraints

use crate::{ResolverConfig, Result, RuntimeDependency, RuntimeMap, RuntimeSpec};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::{debug, trace, warn};
use vx_paths::PathResolver as VxPathResolver;

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

    /// Runtimes that are not supported on the current platform
    pub unsupported_platform_runtimes: Vec<UnsupportedPlatformRuntime>,
}

/// Resolved dependency graph for a runtime execution request.
///
/// Phase 1 keeps this structure intentionally close to `ResolutionResult`.
/// In later phases, we may remove machine-specific fields (like absolute paths)
/// and cache a more stable representation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResolvedGraph {
    /// The primary runtime to execute
    pub runtime: String,

    /// The actual executable to run (may be a bare name if not installed)
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

    /// Runtimes that are not supported on the current platform
    pub unsupported_platform_runtimes: Vec<UnsupportedPlatformRuntime>,
}

impl From<ResolutionResult> for ResolvedGraph {
    fn from(value: ResolutionResult) -> Self {
        Self {
            runtime: value.runtime,
            executable: value.executable,
            command_prefix: value.command_prefix,
            missing_dependencies: value.missing_dependencies,
            install_order: value.install_order,
            runtime_needs_install: value.runtime_needs_install,
            incompatible_dependencies: value.incompatible_dependencies,
            unsupported_platform_runtimes: value.unsupported_platform_runtimes,
        }
    }
}

impl From<ResolvedGraph> for ResolutionResult {
    fn from(value: ResolvedGraph) -> Self {
        Self {
            runtime: value.runtime,
            executable: value.executable,
            command_prefix: value.command_prefix,
            missing_dependencies: value.missing_dependencies,
            install_order: value.install_order,
            runtime_needs_install: value.runtime_needs_install,
            incompatible_dependencies: value.incompatible_dependencies,
            unsupported_platform_runtimes: value.unsupported_platform_runtimes,
        }
    }
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

    /// Check the status of a runtime
    pub fn check_runtime_status(&self, runtime_name: &str) -> RuntimeStatus {
        // Get the runtime specification if known
        let spec = self.runtime_map.get(runtime_name);
        let resolved_name = spec.map(|s| s.name.as_str()).unwrap_or(runtime_name);
        let executable_name = spec.map(|s| s.get_executable()).unwrap_or(runtime_name);

        // For bundled runtimes, we need to look in the parent runtime's directory
        // e.g., rez-env is bundled with rez, so we look in rez's directory for rez-env executable
        let store_dir_name = self.get_store_directory_name(spec, resolved_name);

        // Check vx-managed installation first if preferred
        if self.config.prefer_vx_managed {
            if let Some(status) = self.check_vx_managed(store_dir_name, executable_name) {
                return status;
            }
        }

        // Check system PATH
        if self.config.fallback_to_system {
            let system_status = self.check_system_path(executable_name);
            if system_status.is_available() {
                return system_status;
            }
        }

        // Check vx-managed if not preferred but fallback enabled
        if !self.config.prefer_vx_managed {
            if let Some(status) = self.check_vx_managed(store_dir_name, executable_name) {
                return status;
            }
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
                debug!(
                    "Found vx-managed {} version {} in {} at {}",
                    runtime_name,
                    location.version,
                    location.source,
                    location.path.display()
                );
                Some(RuntimeStatus::VxManaged {
                    version: location.version,
                    path: location.path,
                })
            }
            Ok(None) => {
                debug!("Tool {} not found in vx-managed directories", runtime_name);
                None
            }
            Err(e) => {
                debug!("Error checking vx-managed {}: {}", runtime_name, e);
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

    /// Resolve a runtime for execution
    ///
    /// This method:
    /// 1. Looks up the runtime specification
    /// 2. Checks if the runtime and its dependencies are installed
    /// 3. Returns a resolution result with execution details
    pub fn resolve(&self, runtime_name: &str) -> Result<ResolutionResult> {
        self.resolve_with_version(runtime_name, None)
    }

    /// Resolve a runtime into a serializable dependency graph.
    pub fn resolve_graph(&self, runtime_name: &str) -> Result<ResolvedGraph> {
        Ok(self.resolve(runtime_name)?.into())
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
        if let Some(ver) = version {
            debug!("Resolving runtime: {}@{}", runtime_name, ver);
        } else {
            debug!("Resolving runtime: {}", runtime_name);
        }

        // Get the runtime specification
        let spec = self.runtime_map.get(runtime_name);
        debug!(
            ">>> SPEC CHECK: {} has {} dependencies",
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

        if let Some(spec) = spec {
            // Check each required dependency
            for dep in spec.required_dependencies() {
                let dep_name = dep.provided_by.as_deref().unwrap_or(&dep.runtime_name);

                // Note: Platform compatibility checking for dependencies is done at the CLI layer

                let dep_status = self.check_runtime_status(dep_name);

                debug!(
                    "Checking dependency {} for {} (min: {:?}, max: {:?})",
                    dep_name, runtime_name, dep.min_version, dep.max_version
                );

                match &dep_status {
                    RuntimeStatus::VxManaged { version, .. } => {
                        // Check version constraints
                        let is_compatible = dep.is_version_compatible(version);
                        debug!(
                            "Dependency {} version {} is {} (min: {:?}, max: {:?})",
                            dep_name,
                            version,
                            if is_compatible {
                                "compatible"
                            } else {
                                "INCOMPATIBLE"
                            },
                            dep.min_version,
                            dep.max_version
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
                        {
                            if !dep.is_version_compatible(&system_version) {
                                warn!(
                                    "System {} version {} does not meet constraints for {} (min: {:?}, max: {:?})",
                                    dep_name, system_version, runtime_name, dep.min_version, dep.max_version
                                );
                                incompatible_deps.push(IncompatibleDependency {
                                    runtime_name: dep_name.to_string(),
                                    current_version: Some(system_version),
                                    constraint: dep.clone(),
                                    recommended_version: dep.recommended_version.clone(),
                                });
                            }
                        }
                    }
                    RuntimeStatus::NotInstalled | RuntimeStatus::Unknown => {
                        debug!("Missing dependency: {} (for {})", dep_name, runtime_name);
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
            unsupported_platform_runtimes,
        })
    }

    /// Resolve a runtime into a serializable dependency graph with optional version.
    pub fn resolve_graph_with_version(
        &self,
        runtime_name: &str,
        version: Option<&str>,
    ) -> Result<ResolvedGraph> {
        Ok(self.resolve_with_version(runtime_name, version)?.into())
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
        // Try to find version patterns
        let version_regex = regex::Regex::new(r"v?(\d+\.\d+\.\d+)").ok()?;
        if let Some(captures) = version_regex.captures(output) {
            return Some(captures.get(1)?.as_str().to_string());
        }

        // Try simpler pattern (just major.minor)
        let simple_regex = regex::Regex::new(r"v?(\d+\.\d+)").ok()?;
        if let Some(captures) = simple_regex.captures(output) {
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
                debug!(
                    "Found vx-managed {} version {} at {}",
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
                debug!(
                    "Tool {} version {} not found in vx-managed directories",
                    runtime_name, version
                );
                None
            }
        }
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
