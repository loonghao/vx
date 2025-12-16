//! Runtime resolver - detects and resolves runtime dependencies
//!
//! This module handles:
//! - Detecting installed runtimes (both vx-managed and system)
//! - Resolving runtime dependencies
//! - Determining what needs to be installed

use crate::{ResolverConfig, Result, RuntimeMap, RuntimeSpec};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::{debug, trace};
use vx_paths::PathManager;

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

/// Resolution result for a runtime execution request
#[derive(Debug)]
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
}

/// Runtime resolver that handles dependency detection and resolution
pub struct Resolver {
    /// Runtime map for runtime specifications
    runtime_map: RuntimeMap,

    /// Path manager for vx-managed runtimes
    path_manager: PathManager,

    /// Configuration
    config: ResolverConfig,
}

impl Resolver {
    /// Create a new runtime resolver
    pub fn new(config: ResolverConfig) -> Result<Self> {
        let path_manager = PathManager::new()?;
        Ok(Self {
            runtime_map: RuntimeMap::new(),
            path_manager,
            config,
        })
    }

    /// Create a resolver with a custom runtime map (for testing)
    pub fn with_runtime_map(config: ResolverConfig, runtime_map: RuntimeMap) -> Result<Self> {
        let path_manager = PathManager::new()?;
        Ok(Self {
            runtime_map,
            path_manager,
            config,
        })
    }

    /// Check the status of a runtime
    pub fn check_runtime_status(&self, runtime_name: &str) -> RuntimeStatus {
        // First, check if it's a known runtime
        let spec = match self.runtime_map.get(runtime_name) {
            Some(spec) => spec,
            None => {
                // Unknown runtime - check system PATH only
                return self.check_system_path(runtime_name);
            }
        };

        let executable_name = spec.get_executable();

        // Check vx-managed installation first if preferred
        if self.config.prefer_vx_managed {
            if let Some(status) = self.check_vx_managed(executable_name) {
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
            if let Some(status) = self.check_vx_managed(executable_name) {
                return status;
            }
        }

        RuntimeStatus::NotInstalled
    }

    /// Check if a runtime is installed via vx
    fn check_vx_managed(&self, runtime_name: &str) -> Option<RuntimeStatus> {
        // First check the new store directory (~/.vx/store/<runtime>/<version>)
        if let Some(status) = self.check_store_dir(runtime_name) {
            return Some(status);
        }

        // Fall back to legacy tools directory (~/.vx/tools/<tool>/<version>)
        self.check_tools_dir(runtime_name)
    }

    /// Check the new store directory for installed runtimes
    fn check_store_dir(&self, runtime_name: &str) -> Option<RuntimeStatus> {
        let store_dir = self.path_manager.runtime_store_dir(runtime_name);
        debug!(
            "Checking store directory for {}: {} (exists: {})",
            runtime_name,
            store_dir.display(),
            store_dir.exists()
        );

        let versions = match self.path_manager.list_store_versions(runtime_name) {
            Ok(v) => v,
            Err(e) => {
                debug!("Failed to list store versions for {}: {}", runtime_name, e);
                return None;
            }
        };

        debug!(
            "Found {} versions for {} in store: {:?}",
            versions.len(),
            runtime_name,
            versions
        );

        for version in versions.iter() {
            let version_dir = self.path_manager.version_store_dir(runtime_name, version);
            debug!(
                "Checking version directory: {} (exists: {})",
                version_dir.display(),
                version_dir.exists()
            );

            // Search for the executable in the version directory
            if let Some(exe_path) = self.find_executable_in_dir(&version_dir, runtime_name) {
                debug!(
                    "Found vx-managed {} version {} in store at {}",
                    runtime_name,
                    version,
                    exe_path.display()
                );
                return Some(RuntimeStatus::VxManaged {
                    version: version.clone(),
                    path: exe_path,
                });
            } else {
                debug!(
                    "Executable {} not found in version directory {}",
                    runtime_name,
                    version_dir.display()
                );
            }
        }
        None
    }

    /// Check the legacy tools directory for installed runtimes
    fn check_tools_dir(&self, runtime_name: &str) -> Option<RuntimeStatus> {
        let versions = self.path_manager.list_tool_versions(runtime_name).ok()?;

        if let Some(version) = versions.first() {
            // First try the simple path (e.g., ~/.vx/tools/node/18.17.0/node)
            let simple_path = self
                .path_manager
                .tool_executable_path(runtime_name, version);
            if simple_path.exists() {
                debug!(
                    "Found vx-managed {} version {} at simple path",
                    runtime_name, version
                );
                return Some(RuntimeStatus::VxManaged {
                    version: version.clone(),
                    path: simple_path,
                });
            }

            // If not found, search for the executable in the version directory
            // This handles cases like bun where the exe is in a subdirectory
            let version_dir = self.path_manager.tool_version_dir(runtime_name, version);
            if let Some(exe_path) = self.find_executable_in_dir(&version_dir, runtime_name) {
                debug!(
                    "Found vx-managed {} version {} at {}",
                    runtime_name,
                    version,
                    exe_path.display()
                );
                return Some(RuntimeStatus::VxManaged {
                    version: version.clone(),
                    path: exe_path,
                });
            }
        }
        None
    }

    /// Search for an executable in a directory (recursively, up to 3 levels)
    /// This handles various archive structures:
    /// - Direct: ~/.vx/store/uv/0.9.17/uv
    /// - One level: ~/.vx/store/uv/0.9.17/uv-platform/uv
    /// - Two levels: ~/.vx/store/go/1.25.5/go/bin/go
    fn find_executable_in_dir(&self, dir: &PathBuf, exe_name: &str) -> Option<PathBuf> {
        use std::fs;

        if !dir.exists() {
            return None;
        }

        // Build list of possible executable names in priority order
        // On Windows, .exe and .cmd should be preferred over extensionless files
        // because extensionless files are typically shell scripts
        let possible_names: Vec<String> = if cfg!(windows) {
            vec![
                format!("{}.exe", exe_name),
                format!("{}.cmd", exe_name),
                exe_name.to_string(),
            ]
        } else {
            vec![exe_name.to_string()]
        };

        /// Helper to find the best matching executable from a list of candidates
        /// Returns the one with highest priority (lowest index in possible_names)
        fn find_best_match(candidates: &[PathBuf], possible_names: &[String]) -> Option<PathBuf> {
            for name in possible_names {
                for candidate in candidates {
                    if let Some(file_name) = candidate.file_name().and_then(|n| n.to_str()) {
                        if file_name == name {
                            return Some(candidate.clone());
                        }
                    }
                }
            }
            None
        }

        // Collect all matching files at each level, then pick the best one
        let mut all_candidates: Vec<PathBuf> = Vec::new();

        // Check direct children (level 1)
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();

                // Check if this is a matching executable
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if possible_names.iter().any(|n| n == name) {
                            all_candidates.push(path.clone());
                        }
                    }
                }

                // Check one level deeper (level 2)
                if path.is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file() {
                                if let Some(name) = sub_path.file_name().and_then(|n| n.to_str()) {
                                    if possible_names.iter().any(|n| n == name) {
                                        all_candidates.push(sub_path.clone());
                                    }
                                }
                            }

                            // Check two levels deeper (level 3) - for go/bin/go structure
                            if sub_path.is_dir() {
                                if let Ok(deep_entries) = fs::read_dir(&sub_path) {
                                    for deep_entry in deep_entries.filter_map(|e| e.ok()) {
                                        let deep_path = deep_entry.path();
                                        if deep_path.is_file() {
                                            if let Some(name) =
                                                deep_path.file_name().and_then(|n| n.to_str())
                                            {
                                                if possible_names.iter().any(|n| n == name) {
                                                    all_candidates.push(deep_path);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        find_best_match(&all_candidates, &possible_names)
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
        debug!("Resolving runtime: {}", runtime_name);

        // Get the runtime specification
        let spec = self.runtime_map.get(runtime_name);

        // Check runtime status
        let runtime_status = self.check_runtime_status(runtime_name);

        // Collect missing dependencies
        let mut missing_deps = Vec::new();
        let mut install_order = Vec::new();

        if let Some(spec) = spec {
            // Check each required dependency
            for dep in spec.required_dependencies() {
                let dep_name = dep.provided_by.as_deref().unwrap_or(&dep.runtime_name);
                let dep_status = self.check_runtime_status(dep_name);

                if !dep_status.is_available() {
                    debug!("Missing dependency: {} (for {})", dep_name, runtime_name);
                    missing_deps.push(dep_name.to_string());
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
        })
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
