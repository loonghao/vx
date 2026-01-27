//! Integrated version resolution and environment builder
//!
//! This module provides a unified implementation for:
//! - Version resolution using existing VersionResolver
//! - Path resolution for installed runtimes
//! - Environment variable building (REZ-like)

use crate::{
    provider_env::{ProviderEnvironment, ResolvedVersionInfo},
    version_resolver::VersionResolver,
    Ecosystem, VersionInfo,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use vx_paths::PathManager;

/// Integrated resolver that uses vx's existing infrastructure
pub struct IntegratedVersionResolver {
    /// Path manager for resolving paths
    path_manager: PathManager,
    /// Version resolver for constraint matching
    version_resolver: VersionResolver,
}

impl IntegratedVersionResolver {
    /// Create a new integrated resolver
    pub fn new() -> Result<Self> {
        Ok(Self {
            path_manager: PathManager::new()?,
            version_resolver: VersionResolver::new(),
        })
    }

    /// Resolve a version request and get paths
    ///
    /// This method:
    /// 1. Checks if the version is installed (in platform-specific directory)
    /// 2. If not, tries to resolve the request against available versions
    /// 3. Returns paths for the resolved version
    pub fn resolve_and_get_paths(
        &self,
        runtime_name: &str,
        version_request: &str,
        ecosystem: &Ecosystem,
    ) -> Result<ResolvedVersionInfo> {
        // Use platform-specific directory for installation check
        let platform_store_dir = self.path_manager.platform_store_dir(runtime_name, version_request);

        if platform_store_dir.exists() {
            // Version is installed, return paths
            self.build_resolved_info(runtime_name, version_request, &platform_store_dir)
        } else {
            // Try to resolve version request (if it's a partial version like "3.11")
            self.resolve_version_request(runtime_name, version_request, ecosystem)
        }
    }

    /// Resolve a version request against installed versions
    fn resolve_version_request(
        &self,
        runtime_name: &str,
        version_request: &str,
        ecosystem: &Ecosystem,
    ) -> Result<ResolvedVersionInfo> {
        // Get all installed versions for this runtime
        let store_base = self.path_manager.store_dir().join(runtime_name);

        if !store_base.exists() {
            anyhow::bail!(
                "No versions found for runtime '{}'. Install it first.",
                runtime_name
            );
        }

        let mut available_versions: Vec<VersionInfo> = Vec::new();
        for entry in std::fs::read_dir(&store_base).context("Failed to read store directory")? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(version_str) = entry.file_name().to_str() {
                    if let Some(version) = crate::version_resolver::Version::parse(version_str) {
                        available_versions.push(VersionInfo::new(&version.to_string()));
                    }
                }
            }
        }

        if available_versions.is_empty() {
            anyhow::bail!(
                "No valid versions found for runtime '{}'",
                runtime_name
            );
        }

        // Use VersionResolver to find best match
        if let Some(resolved_version_str) = self.version_resolver.resolve(version_request, &available_versions, ecosystem) {
            let resolved_dir = store_base.join(&resolved_version_str);
            self.build_resolved_info(runtime_name, version_request, &resolved_dir)
        } else {
            anyhow::bail!(
                "Failed to resolve version '{}' for runtime '{}'",
                version_request, runtime_name
            )
        }
    }

    /// Build resolved version info from an installed directory
    fn build_resolved_info(
        &self,
        runtime_name: &str,
        version_request: &str,
        install_dir: &PathBuf,
    ) -> Result<ResolvedVersionInfo> {
        // Determine version from directory name
        let version = install_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid install directory path"))?;

        // Build bin directory path
        let bin_dir = self.detect_bin_dir(runtime_name, install_dir);

        // Build executable path
        let executable_path = self.detect_executable(runtime_name, &bin_dir, install_dir)?;

        Ok(ResolvedVersionInfo::new(
            version.to_string(),
            version_request.to_string(),
            install_dir.clone(),
            executable_path,
            bin_dir,
        ))
    }

    /// Detect the bin directory for a runtime
    fn detect_bin_dir(&self, _runtime_name: &str, install_dir: &PathBuf) -> PathBuf {
        // Common bin directory locations
        let possible_bins = [
            install_dir.join("bin"),
            install_dir.join("Scripts"), // Windows (Python)
            install_dir.clone(), // Some tools have binaries in root
        ];

        for bin_path in &possible_bins {
            if bin_path.exists() {
                return bin_path.clone();
            }
        }

        // Default to "bin"
        install_dir.join("bin")
    }

    /// Detect the executable path for a runtime
    fn detect_executable(&self, runtime_name: &str, bin_dir: &PathBuf, install_dir: &PathBuf) -> Result<PathBuf> {
        // Check in bin_dir
        let exe_in_bin = bin_dir.join(runtime_name);
        if exe_in_bin.exists() {
            return Ok(exe_in_bin);
        }

        // Check with .exe extension (Windows)
        let exe_in_bin_win = bin_dir.join(format!("{}.exe", runtime_name));
        if exe_in_bin_win.exists() {
            return Ok(exe_in_bin_win);
        }

        // Check in install_dir
        let exe_in_root = install_dir.join(runtime_name);
        if exe_in_root.exists() {
            return Ok(exe_in_root);
        }

        let exe_in_root_win = install_dir.join(format!("{}.exe", runtime_name));
        if exe_in_root_win.exists() {
            return Ok(exe_in_root_win);
        }

        // Default to bin_dir/runtime_name
        Ok(bin_dir.join(runtime_name))
    }
}

/// Environment builder for provider environments
pub struct ProviderEnvBuilder {
    /// Integrated resolver
    resolver: IntegratedVersionResolver,
}

impl ProviderEnvBuilder {
    /// Create a new provider environment builder
    pub fn new() -> Result<Self> {
        Ok(Self {
            resolver: IntegratedVersionResolver::new()?,
        })
    }

    /// Build environment for a runtime@version
    pub fn build_for_version(
        &self,
        provider_name: &str,
        runtime_name: &str,
        version_request: &str,
        ecosystem: &Ecosystem,
        manifest_env_vars: Option<&HashMap<String, String>>,
    ) -> Result<ProviderEnvironment> {
        let resolved_version = self.resolver.resolve_and_get_paths(
            runtime_name,
            version_request,
            ecosystem,
        )?;

        let mut env = ProviderEnvironment::new(
            resolved_version,
            provider_name.to_string(),
            runtime_name.to_string(),
        );

        if let Some(vars) = manifest_env_vars {
            env = env.with_manifest_vars(vars.clone());
        }

        Ok(env)
    }

    /// Build PATH from a provider environment
    pub fn build_path_from_env(&self, env: &ProviderEnvironment) -> Vec<PathBuf> {
        env.build_path_prepend()
    }

    /// Build all environment variables from a provider environment
    pub fn build_env_vars_from_env(&self, env: &ProviderEnvironment) -> HashMap<String, String> {
        env.build_env_vars()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_resolved_info() {
        let resolver = IntegratedVersionResolver::new().unwrap();
        let install_dir = PathBuf::from("/tmp/vx/store/python/3.11.11");
        let bin_dir = install_dir.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();

        let result = resolver.build_resolved_info("python", "3.11", &install_dir);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.version, "3.11.11");
        assert_eq!(info.original_request, "3.11");
        assert_eq!(info.install_dir, install_dir);
    }

    #[test]
    fn test_detect_bin_dir() {
        let resolver = IntegratedVersionResolver::new().unwrap();
        let install_dir = PathBuf::from("/tmp/test/python");
        std::fs::create_dir_all(&install_dir).unwrap();

        let bin_dir = install_dir.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();

        let detected = resolver.detect_bin_dir("python", &install_dir);
        assert_eq!(detected, bin_dir);
    }
}
