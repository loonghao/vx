//! Provider Loader
//!
//! This module handles discovery and loading of providers from multiple sources:
//! 1. User local providers (~/.vx/providers/)
//! 2. Environment variable paths (VX_PROVIDERS_PATH)
//! 3. Built-in providers (compiled into vx)
//!
//! User providers take precedence over built-in providers, allowing users to
//! override or extend the default tool configurations.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::{debug, info, warn};
use vx_paths::VxPaths;

use crate::Runtime;
use crate::manifest_runtime::{
    DetectionConfig, InstallStrategy, ManifestDrivenRuntime, ProvidedTool, ProviderSource,
    ScriptType, SystemDepType, SystemDependency, SystemDepsConfig,
};

/// Provider loader configuration
#[derive(Debug, Clone)]
pub struct ProviderLoaderConfig {
    /// Search paths for providers
    pub search_paths: Vec<PathBuf>,
    /// Whether to allow user providers to override built-in
    pub allow_override: bool,
    /// Cache discovered providers
    pub cache_enabled: bool,
}

impl Default for ProviderLoaderConfig {
    fn default() -> Self {
        let mut search_paths = Vec::new();

        // 1. User local providers from VxPaths (highest priority)
        // Uses VX_HOME if set, otherwise ~/.vx
        if let Ok(vx_paths) = VxPaths::new() {
            search_paths.push(vx_paths.providers_dir);
        }

        // 2. Environment variable specified paths (VX_PROVIDERS_PATH)
        if let Ok(paths) = std::env::var("VX_PROVIDERS_PATH") {
            let separator = if cfg!(windows) { ';' } else { ':' };
            for path in paths.split(separator) {
                if !path.is_empty() {
                    search_paths.push(PathBuf::from(path));
                }
            }
        }

        Self {
            search_paths,
            allow_override: true,
            cache_enabled: true,
        }
    }
}

/// Loaded provider information
#[derive(Debug, Clone)]
pub struct LoadedProvider {
    /// Provider name
    pub name: String,
    /// Provider description
    pub description: String,
    /// Source path
    pub source: ProviderSource,
    /// Runtimes provided
    pub runtimes: Vec<ManifestDrivenRuntime>,
}

/// Provider loader
///
/// Discovers and loads providers from multiple sources.
pub struct ProviderLoader {
    /// Configuration
    config: ProviderLoaderConfig,
    /// Loaded providers (cached)
    loaded: HashMap<String, LoadedProvider>,
    /// Runtime name to provider mapping
    runtime_map: HashMap<String, String>,
}

impl ProviderLoader {
    /// Create a new provider loader with default configuration
    pub fn new() -> Self {
        Self::with_config(ProviderLoaderConfig::default())
    }

    /// Create a new provider loader with custom configuration
    pub fn with_config(config: ProviderLoaderConfig) -> Self {
        Self {
            config,
            loaded: HashMap::new(),
            runtime_map: HashMap::new(),
        }
    }

    /// Add a search path
    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) {
        self.config.search_paths.push(path.into());
    }

    /// Discover all manifest-driven providers
    pub fn discover(&mut self) -> Result<Vec<ManifestDrivenRuntime>> {
        let mut all_runtimes = Vec::new();

        for search_path in &self.config.search_paths.clone() {
            if !search_path.exists() {
                debug!("Provider search path does not exist: {:?}", search_path);
                continue;
            }

            debug!("Scanning provider path: {:?}", search_path);

            match std::fs::read_dir(search_path) {
                Ok(entries) => {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let provider_dir = entry.path();
                        if !provider_dir.is_dir() {
                            continue;
                        }

                        let provider_toml = provider_dir.join("provider.toml");
                        if !provider_toml.exists() {
                            continue;
                        }

                        match self.load_provider(&provider_toml) {
                            Ok(provider) => {
                                info!(
                                    "Loaded provider '{}' from {:?} with {} runtimes",
                                    provider.name,
                                    provider_toml,
                                    provider.runtimes.len()
                                );

                                for runtime in &provider.runtimes {
                                    // Check for conflicts
                                    if let Some(existing) = self.runtime_map.get(runtime.name()) {
                                        if self.config.allow_override {
                                            debug!(
                                                "Runtime '{}' from '{}' overrides '{}'",
                                                runtime.name(),
                                                provider.name,
                                                existing
                                            );
                                        } else {
                                            warn!(
                                                "Runtime '{}' already provided by '{}', skipping",
                                                runtime.name(),
                                                existing
                                            );
                                            continue;
                                        }
                                    }

                                    self.runtime_map
                                        .insert(runtime.name().to_string(), provider.name.clone());
                                    all_runtimes.push(runtime.clone());
                                }

                                self.loaded.insert(provider.name.clone(), provider);
                            }
                            Err(e) => {
                                warn!("Failed to load provider from {:?}: {}", provider_toml, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read provider directory {:?}: {}", search_path, e);
                }
            }
        }

        Ok(all_runtimes)
    }

    /// Load a provider from a provider.toml file
    fn load_provider(&self, path: &Path) -> Result<LoadedProvider> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;

        let manifest: toml::Value =
            toml::from_str(&content).with_context(|| format!("Failed to parse {:?}", path))?;

        let provider_table = manifest
            .get("provider")
            .ok_or_else(|| anyhow::anyhow!("Missing [provider] section"))?;

        let name = provider_table
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing provider.name"))?
            .to_string();

        let description = provider_table
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let source_path = path.parent().unwrap().to_path_buf();
        let source = self.determine_source(&source_path);

        // Parse runtimes
        let runtimes = self.parse_runtimes(&manifest, &name, &source)?;

        Ok(LoadedProvider {
            name,
            description,
            source,
            runtimes,
        })
    }

    /// Determine the source type based on path
    fn determine_source(&self, path: &Path) -> ProviderSource {
        // Check if it's in vx providers directory
        if let Ok(vx_paths) = VxPaths::new()
            && path.starts_with(&vx_paths.providers_dir)
        {
            return ProviderSource::UserLocal(path.to_path_buf());
        }

        // Check if it's from VX_PROVIDERS_PATH
        if let Ok(env_paths) = std::env::var("VX_PROVIDERS_PATH") {
            let separator = if cfg!(windows) { ';' } else { ':' };
            for env_path in env_paths.split(separator) {
                if path.starts_with(env_path) {
                    return ProviderSource::EnvPath(path.to_path_buf());
                }
            }
        }

        ProviderSource::UserLocal(path.to_path_buf())
    }

    /// Parse runtimes from manifest
    fn parse_runtimes(
        &self,
        manifest: &toml::Value,
        provider_name: &str,
        source: &ProviderSource,
    ) -> Result<Vec<ManifestDrivenRuntime>> {
        let runtimes_array = match manifest.get("runtimes") {
            Some(toml::Value::Array(arr)) => arr,
            _ => return Ok(Vec::new()),
        };

        let mut runtimes = Vec::new();

        for runtime_value in runtimes_array {
            let name = runtime_value
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Runtime missing name"))?;

            let mut runtime = ManifestDrivenRuntime::new(name, provider_name, source.clone());

            // Description
            if let Some(desc) = runtime_value.get("description").and_then(|v| v.as_str()) {
                runtime = runtime.with_description(desc);
            }

            // Executable
            if let Some(exe) = runtime_value.get("executable").and_then(|v| v.as_str()) {
                runtime = runtime.with_executable(exe);
            }

            // Aliases
            if let Some(aliases) = runtime_value.get("aliases").and_then(|v| v.as_array()) {
                for alias in aliases {
                    if let Some(a) = alias.as_str() {
                        runtime.aliases.push(a.to_string());
                    }
                }
            }

            // Bundled with (for bundled runtimes like npm with node)
            if let Some(bundled) = runtime_value.get("bundled_with").and_then(|v| v.as_str()) {
                runtime = runtime.with_bundled_with(bundled);
            }

            // Detection configuration
            if let Some(detection) = runtime_value.get("detection") {
                runtime.detection = self.parse_detection(detection);
            }

            // System installation strategies
            if let Some(system_install) = runtime_value.get("system_install") {
                if let Some(strategies) =
                    system_install.get("strategies").and_then(|v| v.as_array())
                {
                    for strategy_value in strategies {
                        if let Some(strategy) = self.parse_install_strategy(strategy_value) {
                            runtime.install_strategies.push(strategy);
                        }
                    }
                }

                // Provided tools
                if let Some(provides) = system_install.get("provides").and_then(|v| v.as_array()) {
                    for provide_value in provides {
                        if let Some(provided) = self.parse_provided_tool(provide_value) {
                            runtime.provides.push(provided);
                        }
                    }
                }
            }

            // System dependencies
            if let Some(system_deps) = runtime_value.get("system_deps") {
                runtime.system_deps = Some(self.parse_system_deps(system_deps));
            }

            // RFC 0022: Normalize configuration
            if let Some(normalize) = runtime_value.get("normalize") {
                // Try to deserialize the normalize section using serde
                if let Ok(normalize_config) =
                    normalize.clone().try_into::<vx_manifest::NormalizeConfig>()
                {
                    runtime = runtime.with_normalize(normalize_config);
                }
            }

            // RFC 0018: Mirror configuration
            if let Some(mirrors) = runtime_value.get("mirrors").and_then(|v| v.as_array()) {
                let mut mirror_configs = Vec::new();
                for mirror_value in mirrors {
                    if let Ok(mirror_config) =
                        mirror_value.clone().try_into::<vx_manifest::MirrorConfig>()
                    {
                        mirror_configs.push(mirror_config);
                    }
                }
                if !mirror_configs.is_empty() {
                    runtime = runtime.with_mirrors(mirror_configs);
                }
            }

            runtimes.push(runtime);
        }

        Ok(runtimes)
    }

    /// Parse detection configuration
    fn parse_detection(&self, value: &toml::Value) -> Option<DetectionConfig> {
        let command = value.get("command").and_then(|v| v.as_str())?.to_string();
        let pattern = value.get("pattern").and_then(|v| v.as_str())?.to_string();

        let system_paths = value
            .get("system_paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let env_hints = value
            .get("env_hints")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Some(DetectionConfig {
            command,
            pattern,
            system_paths,
            env_hints,
        })
    }

    /// Parse installation strategy
    fn parse_install_strategy(&self, value: &toml::Value) -> Option<InstallStrategy> {
        let strategy_type = value.get("type").and_then(|v| v.as_str())?;

        let platforms = value
            .get("platforms")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let priority = value
            .get("priority")
            .and_then(|v| v.as_integer())
            .unwrap_or(50) as i32;

        match strategy_type {
            "package_manager" => {
                let manager = value.get("manager").and_then(|v| v.as_str())?.to_string();
                let package = value.get("package").and_then(|v| v.as_str())?.to_string();
                let params = value
                    .get("params")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let install_args = value
                    .get("install_args")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                Some(InstallStrategy::PackageManager {
                    manager,
                    package,
                    params,
                    install_args,
                    priority,
                    platforms,
                })
            }
            "direct_download" => {
                let url = value.get("url").and_then(|v| v.as_str())?.to_string();
                let format = value
                    .get("format")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let executable_path = value
                    .get("executable_path")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                Some(InstallStrategy::DirectDownload {
                    url,
                    format,
                    executable_path,
                    priority,
                    platforms,
                })
            }
            "script" => {
                let url = value.get("url").and_then(|v| v.as_str())?.to_string();
                let script_type_str = value
                    .get("script_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("bash");
                let script_type = match script_type_str {
                    "powershell" | "ps1" => ScriptType::PowerShell,
                    "cmd" | "batch" => ScriptType::Cmd,
                    _ => ScriptType::Bash,
                };
                let args = value
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                Some(InstallStrategy::Script {
                    url,
                    script_type,
                    args,
                    priority,
                    platforms,
                })
            }
            "provided_by" => {
                let provider = value.get("provider").and_then(|v| v.as_str())?.to_string();
                let relative_path = value
                    .get("relative_path")
                    .and_then(|v| v.as_str())?
                    .to_string();

                Some(InstallStrategy::ProvidedBy {
                    provider,
                    relative_path,
                    priority,
                    platforms,
                })
            }
            _ => None,
        }
    }

    /// Parse provided tool
    fn parse_provided_tool(&self, value: &toml::Value) -> Option<ProvidedTool> {
        let name = value.get("name").and_then(|v| v.as_str())?.to_string();
        let relative_path = value
            .get("relative_path")
            .and_then(|v| v.as_str())?
            .to_string();
        let platforms = value
            .get("platforms")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Some(ProvidedTool {
            name,
            relative_path,
            platforms,
        })
    }

    /// Parse system dependencies
    fn parse_system_deps(&self, value: &toml::Value) -> SystemDepsConfig {
        let mut config = SystemDepsConfig::default();

        if let Some(pre_depends) = value.get("pre_depends").and_then(|v| v.as_array()) {
            config.pre_depends = pre_depends
                .iter()
                .filter_map(|v| self.parse_system_dependency(v))
                .collect();
        }

        if let Some(depends) = value.get("depends").and_then(|v| v.as_array()) {
            config.depends = depends
                .iter()
                .filter_map(|v| self.parse_system_dependency(v))
                .collect();
        }

        if let Some(recommends) = value.get("recommends").and_then(|v| v.as_array()) {
            config.recommends = recommends
                .iter()
                .filter_map(|v| self.parse_system_dependency(v))
                .collect();
        }

        if let Some(suggests) = value.get("suggests").and_then(|v| v.as_array()) {
            config.suggests = suggests
                .iter()
                .filter_map(|v| self.parse_system_dependency(v))
                .collect();
        }

        config
    }

    /// Parse a single system dependency
    fn parse_system_dependency(&self, value: &toml::Value) -> Option<SystemDependency> {
        let dep_type_str = value.get("type").and_then(|v| v.as_str())?;
        let dep_type = match dep_type_str {
            "windows_kb" => SystemDepType::WindowsKb,
            "windows_feature" => SystemDepType::WindowsFeature,
            "vc_redist" | "vcredist" => SystemDepType::VcRedist,
            "dotnet" | "dotnet_framework" => SystemDepType::DotNet,
            "package" => SystemDepType::Package,
            "runtime" => SystemDepType::Runtime,
            _ => return None,
        };

        let id = value.get("id").and_then(|v| v.as_str())?.to_string();
        let version = value
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);
        let reason = value
            .get("reason")
            .and_then(|v| v.as_str())
            .map(String::from);
        let platforms = value
            .get("platforms")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let optional = value
            .get("optional")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Some(SystemDependency {
            dep_type,
            id,
            version,
            reason,
            platforms,
            optional,
        })
    }

    /// Get a loaded provider by name
    pub fn get_provider(&self, name: &str) -> Option<&LoadedProvider> {
        self.loaded.get(name)
    }

    /// Get all loaded providers
    pub fn providers(&self) -> impl Iterator<Item = &LoadedProvider> {
        self.loaded.values()
    }

    /// Get the provider for a runtime
    pub fn provider_for_runtime(&self, runtime_name: &str) -> Option<&LoadedProvider> {
        self.runtime_map
            .get(runtime_name)
            .and_then(|provider_name| self.loaded.get(provider_name))
    }

    /// List all available runtimes
    pub fn list_runtimes(&self) -> Vec<(&str, &str)> {
        self.loaded
            .values()
            .flat_map(|p| p.runtimes.iter().map(move |r| (r.name(), p.name.as_str())))
            .collect()
    }
}

impl Default for ProviderLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_provider(dir: &Path, name: &str, content: &str) -> PathBuf {
        let provider_dir = dir.join(name);
        std::fs::create_dir_all(&provider_dir).unwrap();
        let provider_toml = provider_dir.join("provider.toml");
        let mut file = std::fs::File::create(&provider_toml).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        provider_toml
    }

    #[test]
    fn test_load_simple_provider() {
        let temp_dir = TempDir::new().unwrap();

        let content = r#"
[provider]
name = "mytools"
description = "My custom tools"

[[runtimes]]
name = "fd"
description = "A simple, fast alternative to find"
executable = "fd"

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "brew"
package = "fd"
priority = 90
"#;

        let _provider_toml = create_test_provider(temp_dir.path(), "mytools", content);

        let mut loader = ProviderLoader::new();
        loader.config.search_paths = vec![temp_dir.path().to_path_buf()];

        let runtimes = loader.discover().unwrap();

        assert_eq!(runtimes.len(), 1);
        assert_eq!(runtimes[0].name(), "fd");
        assert_eq!(runtimes[0].install_strategies.len(), 1);
    }

    #[test]
    fn test_load_provider_with_system_deps() {
        let temp_dir = TempDir::new().unwrap();

        let content = r#"
[provider]
name = "test"
description = "Test provider"

[[runtimes]]
name = "test-tool"
executable = "test-tool"

[[runtimes.system_deps.pre_depends]]
type = "vc_redist"
id = "vcredist140"
version = ">=14.0"
platforms = ["windows"]
reason = "Required for C++ runtime"

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "test-tool"
priority = 80
platforms = ["windows"]
"#;

        create_test_provider(temp_dir.path(), "test", content);

        let mut loader = ProviderLoader::new();
        loader.config.search_paths = vec![temp_dir.path().to_path_buf()];

        let runtimes = loader.discover().unwrap();

        assert_eq!(runtimes.len(), 1);
        let runtime = &runtimes[0];
        assert!(runtime.system_deps.is_some());

        let deps = runtime.system_deps.as_ref().unwrap();
        assert_eq!(deps.pre_depends.len(), 1);
        assert_eq!(deps.pre_depends[0].dep_type, SystemDepType::VcRedist);
    }

    #[test]
    fn test_multiple_strategies() {
        let temp_dir = TempDir::new().unwrap();

        let content = r#"
[provider]
name = "multi"
description = "Multi-strategy provider"

[[runtimes]]
name = "multi-tool"
executable = "multi-tool"

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "brew"
package = "multi-tool"
priority = 90
platforms = ["macos", "linux"]

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "multi-tool"
priority = 80
platforms = ["windows"]

[[runtimes.system_install.strategies]]
type = "direct_download"
url = "https://example.com/multi-tool-{version}.tar.gz"
format = "tar.gz"
priority = 50
"#;

        create_test_provider(temp_dir.path(), "multi", content);

        let mut loader = ProviderLoader::new();
        loader.config.search_paths = vec![temp_dir.path().to_path_buf()];

        let runtimes = loader.discover().unwrap();

        assert_eq!(runtimes.len(), 1);
        assert_eq!(runtimes[0].install_strategies.len(), 3);
    }
}
