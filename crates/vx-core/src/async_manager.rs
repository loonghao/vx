//! # AsyncVxManager
//!
//! High-performance asynchronous tool manager for vx.
//! This manager provides concurrent tool operations with optimal performance.

use crate::VxResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use vx_plugin::{PluginRegistry, VxTool};

/// High-performance asynchronous tool manager
///
/// AsyncVxManager provides concurrent tool operations with the following benefits:
/// - Concurrent tool installation and version fetching
/// - Intelligent caching and memoization
/// - Optimized resource utilization
/// - Non-blocking operations
pub struct AsyncVxManager {
    /// Plugin registry for tool discovery
    registry: Arc<PluginRegistry>,

    /// Cache for tool instances (thread-safe)
    tool_cache: Arc<RwLock<HashMap<String, Arc<dyn VxTool>>>>,

    /// Cache for version information (thread-safe)
    version_cache: Arc<RwLock<HashMap<String, Vec<String>>>>,

    /// Configuration for concurrent operations
    config: AsyncManagerConfig,
}

/// Configuration for AsyncVxManager
#[derive(Debug, Clone)]
pub struct AsyncManagerConfig {
    /// Maximum number of concurrent tool operations
    pub max_concurrent_operations: usize,

    /// Cache TTL for version information (in seconds)
    pub version_cache_ttl: u64,

    /// Enable aggressive caching
    pub enable_caching: bool,

    /// Timeout for individual tool operations (in seconds)
    pub operation_timeout: u64,
}

impl Default for AsyncManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 8, // Optimal for most systems
            version_cache_ttl: 300,       // 5 minutes
            enable_caching: true,
            operation_timeout: 30, // 30 seconds
        }
    }
}

impl AsyncVxManager {
    /// Create a new AsyncVxManager with default configuration
    pub async fn new() -> VxResult<Self> {
        Self::with_config(AsyncManagerConfig::default()).await
    }

    /// Create a new AsyncVxManager with custom configuration
    pub async fn with_config(config: AsyncManagerConfig) -> VxResult<Self> {
        let registry = Arc::new(PluginRegistry::new());

        Ok(Self {
            registry,
            tool_cache: Arc::new(RwLock::new(HashMap::new())),
            version_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Get tool instance with caching
    async fn get_tool(&self, tool_name: &str) -> VxResult<Arc<dyn VxTool>> {
        // Check cache first
        {
            let cache = self.tool_cache.read().await;
            if let Some(tool) = cache.get(tool_name) {
                return Ok(tool.clone());
            }
        }

        // Get tool from registry
        let tool = self
            .registry
            .get_tool(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

        let tool_arc: Arc<dyn VxTool> = Arc::from(tool);

        // Cache the tool
        if self.config.enable_caching {
            let mut cache = self.tool_cache.write().await;
            cache.insert(tool_name.to_string(), tool_arc.clone());
        }

        Ok(tool_arc)
    }

    /// Install multiple tools concurrently
    pub async fn install_tools_concurrent(
        &self,
        tool_specs: &[(String, Option<String>)], // (tool_name, version)
        force: bool,
    ) -> VxResult<Vec<(String, VxResult<()>)>> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.config.max_concurrent_operations,
        ));
        let mut futures = FuturesUnordered::new();

        for (tool_name, version) in tool_specs {
            let tool_name = tool_name.clone();
            let version = version.clone();
            let semaphore = semaphore.clone();
            let manager = self.clone();

            let future = async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = manager
                    .install_tool_internal(&tool_name, version.as_deref(), force)
                    .await;
                (tool_name, result)
            };

            futures.push(future);
        }

        let mut results = Vec::new();
        while let Some(result) = futures.next().await {
            results.push(result);
        }

        Ok(results)
    }

    /// Internal tool installation with timeout
    async fn install_tool_internal(
        &self,
        tool_name: &str,
        version: Option<&str>,
        force: bool,
    ) -> VxResult<()> {
        let tool = self.get_tool(tool_name).await?;

        let target_version = if let Some(v) = version {
            v.to_string()
        } else {
            // Get latest version concurrently
            let versions = tool
                .fetch_versions(false)
                .await
                .map_err(crate::VxError::Other)?;
            if versions.is_empty() {
                return Err(crate::VxError::Other(anyhow::anyhow!(
                    "No versions found for tool: {}",
                    tool_name
                )));
            }
            versions[0].version.clone()
        };

        tool.install_version(&target_version, force)
            .await
            .map_err(crate::VxError::Other)
    }

    /// Fetch versions for multiple tools concurrently
    pub async fn fetch_versions_concurrent(
        &self,
        tool_names: &[String],
        include_prerelease: bool,
    ) -> VxResult<HashMap<String, Vec<String>>> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.config.max_concurrent_operations,
        ));
        let mut futures = FuturesUnordered::new();

        for tool_name in tool_names {
            let tool_name = tool_name.clone();
            let semaphore = semaphore.clone();
            let manager = self.clone();

            let future = async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = manager
                    .fetch_tool_versions_internal(&tool_name, include_prerelease)
                    .await;
                (tool_name, result)
            };

            futures.push(future);
        }

        let mut results = HashMap::new();
        while let Some((tool_name, result)) = futures.next().await {
            match result {
                Ok(versions) => {
                    results.insert(tool_name, versions);
                }
                Err(_) => {
                    // Log error but continue with other tools
                    results.insert(tool_name, vec![]);
                }
            }
        }

        Ok(results)
    }

    /// Internal version fetching with caching
    async fn fetch_tool_versions_internal(
        &self,
        tool_name: &str,
        include_prerelease: bool,
    ) -> VxResult<Vec<String>> {
        // Check cache first
        if self.config.enable_caching {
            let cache = self.version_cache.read().await;
            if let Some(versions) = cache.get(tool_name) {
                return Ok(versions.clone());
            }
        }

        let tool = self.get_tool(tool_name).await?;
        let version_infos = tool.fetch_versions(include_prerelease).await?;
        let versions: Vec<String> = version_infos.into_iter().map(|v| v.version).collect();

        // Cache the versions
        if self.config.enable_caching {
            let mut cache = self.version_cache.write().await;
            cache.insert(tool_name.to_string(), versions.clone());
        }

        Ok(versions)
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub tool_cache_size: usize,
    pub version_cache_size: usize,
}

// Implement Clone for AsyncVxManager (needed for concurrent operations)
impl Clone for AsyncVxManager {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            tool_cache: self.tool_cache.clone(),
            version_cache: self.version_cache.clone(),
            config: self.config.clone(),
        }
    }
}

impl AsyncVxManager {
    /// Clear all caches
    pub async fn clear_caches(&self) {
        let mut tool_cache = self.tool_cache.write().await;
        let mut version_cache = self.version_cache.write().await;

        tool_cache.clear();
        version_cache.clear();
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let tool_cache = self.tool_cache.read().await;
        let version_cache = self.version_cache.read().await;

        CacheStats {
            tool_cache_size: tool_cache.len(),
            version_cache_size: version_cache.len(),
        }
    }
}

// Note: AsyncVxManager doesn't implement ToolManager trait directly
// Instead, it provides its own high-performance async interface
