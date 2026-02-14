//! Provider environment and version resolution
//!
//! This module provides traits and implementations for:
//! - Version resolution and caching at provider level
//! - Environment variable building (inspired by REZ)
//! - Path resolution for resolved versions

use crate::{Ecosystem, VersionInfo, context::RuntimeContext};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Version fetcher function type for resolving versions
pub type VersionFetcherFn = Box<dyn Fn(&str, &[VersionInfo]) -> Option<String> + Send + Sync>;

/// Resolved version information with paths
#[derive(Debug, Clone)]
pub struct ResolvedVersionInfo {
    /// Full version string (e.g., "3.11.11")
    pub version: String,
    /// Original request (e.g., "3.11")
    pub original_request: String,
    /// Installation directory path (ROOT - where the runtime files are)
    pub install_dir: PathBuf,
    /// Base directory path (BASE - version directory without platform)
    pub base_dir: PathBuf,
    /// Executable path
    pub executable_path: PathBuf,
    /// Binary directory path (BIN - for PATH)
    pub bin_dir: PathBuf,
    /// All installed versions of this runtime
    pub all_versions: Vec<String>,
    /// Additional environment variables specific to this runtime
    pub env_vars: HashMap<String, String>,
}

impl ResolvedVersionInfo {
    /// Create a new resolved version info
    pub fn new(
        version: String,
        original_request: String,
        install_dir: PathBuf,
        executable_path: PathBuf,
        bin_dir: PathBuf,
    ) -> Self {
        Self {
            version,
            original_request,
            base_dir: install_dir.clone(), // Default: same as install_dir
            install_dir,
            executable_path,
            bin_dir,
            all_versions: Vec::new(),
            env_vars: HashMap::new(),
        }
    }

    /// Create with separate base directory
    pub fn with_base_dir(mut self, base_dir: PathBuf) -> Self {
        self.base_dir = base_dir;
        self
    }

    /// Set all installed versions
    pub fn with_all_versions(mut self, versions: Vec<String>) -> Self {
        self.all_versions = versions;
        self
    }

    /// Add an environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables
    pub fn with_env_vars(mut self, vars: HashMap<String, String>) -> Self {
        self.env_vars.extend(vars);
        self
    }
}

/// Environment configuration for a resolved version (inspired by REZ)
#[derive(Debug, Clone)]
pub struct ProviderEnvironment {
    /// Resolved version info
    pub version_info: ResolvedVersionInfo,
    /// Provider name (for VX_<PROVIDER>_* vars)
    pub provider_name: String,
    /// Runtime name
    pub runtime_name: String,
    /// Additional environment variables from manifest
    pub manifest_env_vars: HashMap<String, String>,
    /// PATH entries to prepend (in order)
    pub path_entries: Vec<PathBuf>,
}

impl ProviderEnvironment {
    /// Create a new provider environment
    pub fn new(
        version_info: ResolvedVersionInfo,
        provider_name: String,
        runtime_name: String,
    ) -> Self {
        Self {
            version_info,
            provider_name,
            runtime_name,
            manifest_env_vars: HashMap::new(),
            path_entries: vec![],
        }
    }

    /// Add manifest environment variables
    pub fn with_manifest_vars(mut self, vars: HashMap<String, String>) -> Self {
        self.manifest_env_vars.extend(vars);
        self
    }

    /// Add PATH entries (they will be prepended in order)
    pub fn with_path_entries(mut self, mut entries: Vec<PathBuf>) -> Self {
        entries.reverse(); // Reverse so first in list gets prepended first
        self.path_entries.extend(entries);
        self
    }

    /// Build all environment variables for this provider
    ///
    /// This creates REZ-like environment variables:
    /// - VX_<PROVIDER>_ROOT - Root installation directory
    /// - VX_<PROVIDER>_BASE - Base version directory (without platform)
    /// - VX_<PROVIDER>_BIN - Binary directory (for PATH)
    /// - VX_<PROVIDER>_VERSION - Current version
    /// - VX_<PROVIDER>_VERSIONS - All installed versions (separator-joined)
    /// - VX_<PROVIDER>_ORIGINAL_REQUEST - Original version request
    /// - Plus any manifest-specific vars
    pub fn build_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        let provider_upper = self.provider_name.to_uppercase().replace('-', "_");
        let runtime_upper = self.runtime_name.to_uppercase().replace('-', "_");
        let sep = if cfg!(windows) { ";" } else { ":" };

        // REZ-like environment variables for provider
        env.insert(
            format!("VX_{}_ROOT", provider_upper),
            self.version_info.install_dir.display().to_string(),
        );
        env.insert(
            format!("VX_{}_BASE", provider_upper),
            self.version_info.base_dir.display().to_string(),
        );
        env.insert(
            format!("VX_{}_BIN", provider_upper),
            self.version_info.bin_dir.display().to_string(),
        );
        env.insert(
            format!("VX_{}_VERSION", provider_upper),
            self.version_info.version.clone(),
        );
        env.insert(
            format!("VX_{}_VERSIONS", provider_upper),
            self.version_info.all_versions.join(sep),
        );
        env.insert(
            format!("VX_{}_ORIGINAL_REQUEST", provider_upper),
            self.version_info.original_request.clone(),
        );

        // Runtime-specific variables (when runtime name differs from provider name)
        if self.runtime_name != self.provider_name {
            env.insert(
                format!("VX_{}_ROOT", runtime_upper),
                self.version_info.install_dir.display().to_string(),
            );
            env.insert(
                format!("VX_{}_BASE", runtime_upper),
                self.version_info.base_dir.display().to_string(),
            );
            env.insert(
                format!("VX_{}_BIN", runtime_upper),
                self.version_info.bin_dir.display().to_string(),
            );
            env.insert(
                format!("VX_{}_VERSION", runtime_upper),
                self.version_info.version.clone(),
            );
            env.insert(
                format!("VX_{}_VERSIONS", runtime_upper),
                self.version_info.all_versions.join(sep),
            );
        }

        // Manifest environment variables
        for (key, value) in &self.manifest_env_vars {
            // Expand {install_dir}, {base_dir}, {bin_dir}, and {version} placeholders
            let expanded = value
                .replace(
                    "{install_dir}",
                    &self.version_info.install_dir.display().to_string(),
                )
                .replace(
                    "{root}",
                    &self.version_info.install_dir.display().to_string(),
                )
                .replace(
                    "{base_dir}",
                    &self.version_info.base_dir.display().to_string(),
                )
                .replace("{base}", &self.version_info.base_dir.display().to_string())
                .replace(
                    "{bin_dir}",
                    &self.version_info.bin_dir.display().to_string(),
                )
                .replace("{bin}", &self.version_info.bin_dir.display().to_string())
                .replace("{version}", &self.version_info.version)
                .replace("{runtime}", &self.runtime_name)
                .replace("{provider}", &self.provider_name);

            env.insert(key.clone(), expanded);
        }

        // Runtime-specific env vars from version info
        for (key, value) in &self.version_info.env_vars {
            env.insert(key.clone(), value.clone());
        }

        env
    }

    /// Build PATH modification (prepend entries)
    pub fn build_path_prepend(&self) -> Vec<PathBuf> {
        let mut path_entries = self.path_entries.clone();
        path_entries.push(self.version_info.bin_dir.clone());
        path_entries
    }
}

/// Trait for provider-level version resolution and environment building
#[async_trait::async_trait]
pub trait ProviderEnvironmentResolver: Send + Sync {
    /// Provider name
    fn provider_name(&self) -> &str;

    /// Ecosystem for this provider
    fn ecosystem(&self) -> Ecosystem;

    /// Resolve a version request to a full version
    ///
    /// This method:
    /// 1. Parses the version request (e.g., "3.11" -> "3.11.11")
    /// 2. Checks cache for existing resolution
    /// 3. Fetches available versions if not cached
    /// 4. Resolves to best matching version
    /// 5. Caches the result
    async fn resolve_version(
        &self,
        runtime_name: &str,
        version_request: &str,
        ctx: &RuntimeContext,
    ) -> Result<ResolvedVersionInfo>;

    /// Build environment for a resolved version
    ///
    /// This creates a ProviderEnvironment with:
    /// - Resolved version info
    /// - REZ-like environment variables (VX_<PROVIDER>_ROOT, etc.)
    /// - PATH entries to prepend
    /// - Manifest-specific environment variables
    fn build_environment(
        &self,
        resolved_version: &ResolvedVersionInfo,
        manifest_env_vars: Option<&HashMap<String, String>>,
    ) -> ProviderEnvironment;

    /// Get the installation directory for a specific version
    fn get_install_dir(&self, version: &str) -> Result<PathBuf>;

    /// Get the executable path for a specific version
    fn get_executable_path(&self, version: &str, runtime_name: &str) -> Result<PathBuf>;

    /// Get the bin directory for PATH
    fn get_bin_dir(&self, version: &str) -> Result<PathBuf>;
}

/// Simple version resolver with caching
pub struct VersionResolverCache {
    /// Cache of resolved versions
    cache: HashMap<String, ResolvedVersionInfo>,
    #[allow(dead_code)]
    /// Cache TTL (for future use)
    ttl: Duration,
    /// Version resolution function
    version_fetcher: VersionFetcherFn,
}

impl VersionResolverCache {
    /// Create a new version resolver cache
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            ttl,
            version_fetcher: Box::new(|_request, _available| None),
        }
    }

    /// Set the version fetcher function
    pub fn with_fetcher(
        mut self,
        fetcher: impl Fn(&str, &[VersionInfo]) -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        self.version_fetcher = Box::new(fetcher);
        self
    }

    /// Get cached version if not expired
    pub fn get(&self, key: &str) -> Option<&ResolvedVersionInfo> {
        self.cache.get(key)
    }

    /// Cache a resolved version
    pub fn set(&mut self, key: String, value: ResolvedVersionInfo) {
        self.cache.insert(key, value);
    }

    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Resolve a version request with caching
    pub fn resolve(
        &mut self,
        runtime_name: &str,
        version_request: &str,
        available_versions: &[VersionInfo],
    ) -> Result<ResolvedVersionInfo> {
        let cache_key = format!("{}@{}", runtime_name, version_request);

        // Check cache first
        if let Some(cached) = self.get(&cache_key) {
            tracing::debug!(
                runtime = %runtime_name,
                version_request = %version_request,
                resolved_version = %cached.version,
                "Version resolved from cache"
            );
            return Ok(cached.clone());
        }

        // Resolve using fetcher
        let resolved_version = (self.version_fetcher)(version_request, available_versions)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to resolve version '{}' for runtime '{}'",
                    version_request,
                    runtime_name
                )
            })?;

        tracing::debug!(
            runtime = %runtime_name,
            version_request = %version_request,
            resolved_version = %resolved_version,
            "Version resolved from available versions"
        );

        // TODO: Build actual paths based on resolved version
        // For now, create placeholder paths
        let install_dir =
            PathBuf::from(format!("/.vx/store/{}/{}", runtime_name, resolved_version));
        let bin_dir = install_dir.join("bin");
        let executable_path = bin_dir.join(runtime_name);

        let version_info = ResolvedVersionInfo::new(
            resolved_version.clone(),
            version_request.to_string(),
            install_dir,
            executable_path,
            bin_dir,
        );

        // Cache the result
        self.set(cache_key, version_info.clone());

        Ok(version_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_environment_env_vars() {
        let version_info = ResolvedVersionInfo::new(
            "3.11.11".to_string(),
            "3.11".to_string(),
            PathBuf::from("/.vx/store/python/3.11.11"),
            PathBuf::from("/.vx/store/python/3.11.11/bin/python"),
            PathBuf::from("/.vx/store/python/3.11.11/bin"),
        );

        let env =
            ProviderEnvironment::new(version_info, "python".to_string(), "python".to_string());

        let vars = env.build_env_vars();

        // Check REZ-like variables
        assert_eq!(
            vars.get("VX_PYTHON_ROOT"),
            Some(&"/.vx/store/python/3.11.11".to_string())
        );
        assert_eq!(vars.get("VX_PYTHON_VERSION"), Some(&"3.11.11".to_string()));
        assert_eq!(
            vars.get("VX_PYTHON_ORIGINAL_REQUEST"),
            Some(&"3.11".to_string())
        );
    }

    #[test]
    fn test_provider_environment_manifest_vars_expansion() {
        let version_info = ResolvedVersionInfo::new(
            "3.11.11".to_string(),
            "3.11".to_string(),
            PathBuf::from("/.vx/store/python/3.11.11"),
            PathBuf::from("/.vx/store/python/3.11.11/bin/python"),
            PathBuf::from("/.vx/store/python/3.11.11/bin"),
        );

        let mut manifest_vars = HashMap::new();
        manifest_vars.insert("PYTHONHOME".to_string(), "{install_dir}".to_string());
        manifest_vars.insert(
            "MY_TOOL_ROOT".to_string(),
            "{install_dir}/tools".to_string(),
        );

        let env =
            ProviderEnvironment::new(version_info, "python".to_string(), "python".to_string())
                .with_manifest_vars(manifest_vars);

        let vars = env.build_env_vars();

        // Check placeholder expansion
        assert_eq!(
            vars.get("PYTHONHOME"),
            Some(&"/.vx/store/python/3.11.11".to_string())
        );
        assert_eq!(
            vars.get("MY_TOOL_ROOT"),
            Some(&"/.vx/store/python/3.11.11/tools".to_string())
        );
    }
}
