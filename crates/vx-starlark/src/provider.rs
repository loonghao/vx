//! StarlarkProvider - The main interface for loading and executing Starlark provider scripts
//!
//! This module implements the core functionality for:
//! - Loading and parsing provider.star files
//! - Executing provider functions
//! - Caching compiled bytecode
//! - Providing a trait-based interface compatible with vx's Provider system

use crate::context::{InstallResult, ProviderContext, VersionInfo};
use crate::engine::StarlarkEngine;
use crate::error::{Error, Result};
use crate::sandbox::SandboxConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Provider metadata parsed from the script
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderMeta {
    /// Provider name
    pub name: String,
    /// Provider description
    #[serde(default)]
    pub description: String,
    /// Provider version
    #[serde(default = "default_version")]
    pub version: String,
    /// Provider homepage
    #[serde(default)]
    pub homepage: Option<String>,
    /// Provider repository
    #[serde(default)]
    pub repository: Option<String>,
    /// Platform constraints (os: ["windows", "linux"])
    #[serde(default)]
    pub platforms: Option<HashMap<String, Vec<String>>>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Runtime metadata parsed from the script
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeMeta {
    /// Runtime name
    pub name: String,
    /// Runtime description
    #[serde(default)]
    pub description: String,
    /// Executable name
    pub executable: String,
    /// Aliases
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Priority
    #[serde(default = "default_priority")]
    pub priority: u32,
}

fn default_priority() -> u32 {
    100
}

/// A loaded Starlark provider
#[derive(Debug, Clone)]
pub struct StarlarkProvider {
    /// Path to the provider script
    script_path: PathBuf,

    /// Provider metadata
    meta: ProviderMeta,

    /// Runtime definitions
    runtimes: Vec<RuntimeMeta>,

    /// Sandbox configuration
    sandbox: SandboxConfig,

    /// VX home directory
    vx_home: PathBuf,

    /// Cached script content (for engine execution)
    script_content: Arc<String>,

    /// Cached bytecode (for performance)
    #[allow(dead_code)]
    bytecode_cache: Option<Arc<Vec<u8>>>,
}

/// Cache for loaded providers
type ProviderCache = Arc<RwLock<HashMap<PathBuf, StarlarkProvider>>>;

/// Global provider cache
static PROVIDER_CACHE: once_cell::sync::Lazy<ProviderCache> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

impl StarlarkProvider {
    /// Load a Starlark provider from a file
    ///
    /// This will:
    /// 1. Check the cache for a previously loaded provider
    /// 2. Parse the script to extract metadata
    /// 3. Compile the script to bytecode
    /// 4. Cache the result for future use
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Check cache first
        {
            let cache = PROVIDER_CACHE.read().await;
            if let Some(provider) = cache.get(&path) {
                debug!("Using cached provider: {:?}", path);
                return Ok(provider.clone());
            }
        }

        // Check if file exists
        if !path.exists() {
            return Err(Error::ScriptNotFound(path));
        }

        // Read the script
        let content = std::fs::read_to_string(&path)?;
        debug!("Loading Starlark provider from: {:?}", path);

        // Parse metadata from the script
        let (meta, runtimes) = Self::parse_metadata(&content)?;

        // Create the provider
        let vx_home = vx_paths::VxPaths::new()
            .map(|p| p.base_dir)
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".vx"));
        let provider = Self {
            script_path: path.clone(),
            meta,
            runtimes,
            sandbox: SandboxConfig::default(),
            vx_home,
            script_content: Arc::new(content),
            bytecode_cache: None,
        };

        // Cache the provider
        {
            let mut cache = PROVIDER_CACHE.write().await;
            cache.insert(path, provider.clone());
        }

        info!("Loaded Starlark provider: {}", provider.meta.name);
        Ok(provider)
    }

    /// Load a provider with custom sandbox configuration
    pub async fn load_with_sandbox(path: impl AsRef<Path>, sandbox: SandboxConfig) -> Result<Self> {
        let mut provider = Self::load(path).await?;
        provider.sandbox = sandbox;
        Ok(provider)
    }

    /// Get the Starlark engine for this provider
    fn engine(&self) -> StarlarkEngine {
        StarlarkEngine::new()
    }

    /// Parse metadata from a Starlark script
    ///
    /// Looks for:
    /// - `name()` function
    /// - `description()` function
    /// - `runtimes` variable
    fn parse_metadata(content: &str) -> Result<(ProviderMeta, Vec<RuntimeMeta>)> {
        // Simple parsing for metadata
        // In a full implementation, we would use the Starlark parser

        let mut meta = ProviderMeta {
            name: "unknown".to_string(),
            description: String::new(),
            version: "1.0.0".to_string(),
            homepage: None,
            repository: None,
            platforms: None,
        };

        let mut runtimes: Vec<RuntimeMeta> = Vec::new();

        // Parse name() function
        for line in content.lines() {
            let line = line.trim();

            // Parse name()
            if line.starts_with("def name()") {
                if let Some(value) = Self::extract_string_return(line, content) {
                    meta.name = value;
                }
            }

            // Parse description()
            if line.starts_with("def description()") {
                if let Some(value) = Self::extract_string_return(line, content) {
                    meta.description = value;
                }
            }
        }

        // For now, create a default runtime with the provider name
        if runtimes.is_empty() {
            runtimes.push(RuntimeMeta {
                name: meta.name.clone(),
                description: meta.description.clone(),
                executable: meta.name.clone(),
                aliases: vec![],
                priority: 100,
            });
        }

        Ok((meta, runtimes))
    }

    /// Extract string return value from a function
    fn extract_string_return(_func_line: &str, _content: &str) -> Option<String> {
        // Simple extraction - look for return "value" in the next few lines
        // This is a simplified implementation
        None
    }

    /// Get the provider name
    pub fn name(&self) -> &str {
        &self.meta.name
    }

    /// Get the provider description
    pub fn description(&self) -> &str {
        &self.meta.description
    }

    /// Get the provider metadata
    pub fn meta(&self) -> &ProviderMeta {
        &self.meta
    }

    /// Get the runtime definitions
    pub fn runtimes(&self) -> &[RuntimeMeta] {
        &self.runtimes
    }

    /// Get the script path
    pub fn script_path(&self) -> &Path {
        &self.script_path
    }

    // === Provider Functions ===

    /// Call the `fetch_versions` function
    pub async fn fetch_versions(&self) -> Result<Vec<VersionInfo>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_sandbox(self.sandbox.clone());

        // Execute the function
        // In a full implementation, this would use the Starlark evaluator
        self.execute_fetch_versions(&ctx).await
    }

    /// Call the `install` function
    pub async fn install(&self, version: &str) -> Result<InstallResult> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        self.execute_install(&ctx, version).await
    }

    /// Call the `prepare_environment` function
    pub async fn prepare_environment(&self, version: &str) -> Result<HashMap<String, String>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        self.execute_prepare_environment(&ctx, version).await
    }

    // === Internal Execution Methods ===

    /// Execute fetch_versions function using the Starlark engine
    async fn execute_fetch_versions(&self, ctx: &ProviderContext) -> Result<Vec<VersionInfo>> {
        let engine = self.engine();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "fetch_versions",
            ctx,
            &[],
        );

        match result {
            Ok(json) => {
                // Parse JSON array of version dicts
                if let Some(arr) = json.as_array() {
                    let versions = arr
                        .iter()
                        .filter_map(|v| {
                            let version = v.get("version")?.as_str()?.to_string();
                            Some(VersionInfo {
                                version,
                                lts: v.get("lts").and_then(|l| l.as_bool()).unwrap_or(false),
                                stable: v.get("stable").and_then(|s| s.as_bool()).unwrap_or(true),
                                date: v.get("date").and_then(|d| d.as_str()).map(|s| s.to_string()),
                            })
                        })
                        .collect();
                    Ok(versions)
                } else {
                    Ok(vec![])
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                warn!(
                    provider = %self.meta.name,
                    "fetch_versions() not found in provider script"
                );
                Ok(vec![])
            }
            Err(e) => Err(e),
        }
    }

    /// Execute install function using the Starlark engine
    async fn execute_install(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<InstallResult> {
        let engine = self.engine();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "install",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );

        match result {
            Ok(json) => {
                let success = json.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                let install_path = json
                    .get("path")
                    .and_then(|p| p.as_str())
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| ctx.paths.install_dir(version));

                if success {
                    Ok(InstallResult::success(install_path))
                } else {
                    let msg = json
                        .get("error")
                        .and_then(|e| e.as_str())
                        .unwrap_or("Installation failed")
                        .to_string();
                    Ok(InstallResult::failure(msg))
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                // install() is optional - use default Rust installer
                debug!(
                    provider = %self.meta.name,
                    "install() not found, using default installer"
                );
                Ok(InstallResult::failure("No install() function defined"))
            }
            Err(e) => Err(e),
        }
    }

    /// Execute prepare_environment function using the Starlark engine
    async fn execute_prepare_environment(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<HashMap<String, String>> {
        let engine = self.engine();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "prepare_environment",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );

        match result {
            Ok(json) => {
                if let Some(obj) = json.as_object() {
                    Ok(obj
                        .iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect())
                } else {
                    Ok(HashMap::new())
                }
            }
            Err(Error::FunctionNotFound { .. }) => Ok(HashMap::new()),
            Err(e) => Err(e),
        }
    }

    // === Cache Management ===

    /// Clear the provider cache
    pub async fn clear_cache() {
        let mut cache = PROVIDER_CACHE.write().await;
        cache.clear();
        info!("Cleared Starlark provider cache");
    }

    /// Get cache statistics
    pub async fn cache_stats() -> (usize, usize) {
        let cache = PROVIDER_CACHE.read().await;
        (cache.len(), cache.values().map(|p| p.runtimes.len()).sum())
    }
}

/// Detect if a path is a Starlark provider
pub fn is_starlark_provider(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "star")
        .unwrap_or(false)
}

/// Check if a directory contains a Starlark provider
pub fn has_starlark_provider(dir: &Path) -> bool {
    dir.join("provider.star").exists()
}

/// Check if a directory contains a TOML provider
pub fn has_toml_provider(dir: &Path) -> bool {
    dir.join("provider.toml").exists()
}

/// Determine provider format for a directory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderFormat {
    /// Starlark provider (provider.star)
    Starlark,
    /// TOML provider (provider.toml)
    Toml,
    /// No provider found
    None,
}

impl ProviderFormat {
    /// Detect the provider format for a directory
    pub fn detect(dir: &Path) -> Self {
        if has_starlark_provider(dir) {
            Self::Starlark
        } else if has_toml_provider(dir) {
            Self::Toml
        } else {
            Self::None
        }
    }

    /// Get the provider filename
    pub fn filename(&self) -> Option<&'static str> {
        match self {
            Self::Starlark => Some("provider.star"),
            Self::Toml => Some("provider.toml"),
            Self::None => None,
        }
    }
}
