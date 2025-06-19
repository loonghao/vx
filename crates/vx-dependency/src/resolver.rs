//! High-level dependency resolver with caching and optimization

use crate::{
    graph::{DependencyGraph, ResolutionResult},
    types::*,
    Error, Result,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// High-level dependency resolver
pub struct DependencyResolver {
    /// Dependency graph
    graph: Arc<RwLock<DependencyGraph>>,
    /// Tool registry
    tool_registry: Arc<RwLock<HashMap<String, ToolSpec>>>,
    /// Resolution cache
    resolution_cache: Arc<RwLock<HashMap<String, CachedResolution>>>,
    /// Availability checker
    availability_checker: Option<Arc<dyn AvailabilityChecker>>,
    /// Configuration
    options: ResolutionOptions,
}

/// Cached resolution result
#[derive(Debug, Clone)]
struct CachedResolution {
    /// Resolution result
    result: ResolutionResult,
    /// Cache timestamp
    cached_at: Instant,
    /// Cache TTL
    ttl: Duration,
}

/// Options for dependency resolution
#[derive(Debug, Clone)]
pub struct ResolutionOptions {
    /// Whether to include optional dependencies
    pub include_optional: bool,
    /// Whether to include development dependencies
    pub include_dev: bool,
    /// Maximum resolution depth (to prevent infinite recursion)
    pub max_depth: usize,
    /// Cache TTL for resolutions
    pub cache_ttl: Duration,
    /// Whether to enable parallel resolution
    pub enable_parallel: bool,
    /// Platform filter (only include dependencies for this platform)
    pub platform_filter: Option<String>,
    /// Whether to allow prerelease versions
    pub allow_prerelease: bool,
}

/// Trait for checking tool availability
#[async_trait::async_trait]
pub trait AvailabilityChecker: Send + Sync {
    /// Check if a tool is available
    async fn is_available(&self, tool_name: &str) -> Result<bool>;

    /// Get installed version of a tool
    async fn get_version(&self, tool_name: &str) -> Result<Option<String>>;

    /// Get tool installation path
    async fn get_path(&self, tool_name: &str) -> Result<Option<String>>;
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Self {
        Self {
            graph: Arc::new(RwLock::new(DependencyGraph::new())),
            tool_registry: Arc::new(RwLock::new(HashMap::new())),
            resolution_cache: Arc::new(RwLock::new(HashMap::new())),
            availability_checker: None,
            options: ResolutionOptions::default(),
        }
    }

    /// Create a resolver with custom options
    pub fn with_options(options: ResolutionOptions) -> Self {
        Self {
            options,
            ..Self::new()
        }
    }

    /// Set availability checker
    pub fn with_availability_checker(mut self, checker: Arc<dyn AvailabilityChecker>) -> Self {
        self.availability_checker = Some(checker);
        self
    }

    /// Register a tool specification
    pub async fn register_tool(&self, tool_spec: ToolSpec) -> Result<()> {
        let tool_name = tool_spec.name.clone();

        // Add to registry
        {
            let mut registry = self.tool_registry.write().await;
            registry.insert(tool_name.clone(), tool_spec.clone());
        }

        // Add to graph
        {
            let mut graph = self.graph.write().await;
            graph.add_tool(tool_spec)?;
        }

        // Update availability if checker is available
        if let Some(checker) = &self.availability_checker {
            let available = checker.is_available(&tool_name).await.unwrap_or(false);
            let version = if available {
                checker.get_version(&tool_name).await.unwrap_or(None)
            } else {
                None
            };

            let mut graph = self.graph.write().await;
            graph.set_tool_available(&tool_name, available, version);
        }

        Ok(())
    }

    /// Register multiple tools
    pub async fn register_tools(&self, tools: Vec<ToolSpec>) -> Result<()> {
        for tool in tools {
            self.register_tool(tool).await?;
        }
        Ok(())
    }

    /// Resolve dependencies for a tool
    pub async fn resolve(&self, tool_name: &str) -> Result<ResolutionResult> {
        // Check cache first
        if let Some(cached) = self.get_cached_resolution(tool_name).await {
            if cached.cached_at.elapsed() < cached.ttl {
                return Ok(cached.result);
            }
        }

        // Perform resolution
        let result = self.resolve_uncached(tool_name).await?;

        // Cache result
        self.cache_resolution(tool_name, result.clone()).await;

        Ok(result)
    }

    /// Resolve dependencies without caching
    async fn resolve_uncached(&self, tool_name: &str) -> Result<ResolutionResult> {
        // Ensure tool is registered
        if !self.is_tool_registered(tool_name).await {
            return Err(Error::ToolNotFound {
                tool: tool_name.to_string(),
            });
        }

        // Update availability information
        self.update_availability().await?;

        // Perform resolution using the graph
        let mut graph = self.graph.write().await;
        let mut result = graph.resolve_dependencies(tool_name)?;

        // Filter based on options
        self.filter_resolution(&mut result).await;

        Ok(result)
    }

    /// Resolve dependencies for multiple tools
    pub async fn resolve_multiple(&self, tool_names: &[String]) -> Result<ResolutionResult> {
        let mut combined_result = ResolutionResult {
            install_order: Vec::new(),
            missing_tools: Vec::new(),
            available_tools: Vec::new(),
            circular_dependencies: Vec::new(),
            version_conflicts: Vec::new(),
        };

        // Resolve each tool and combine results
        for tool_name in tool_names {
            let result = self.resolve(tool_name).await?;

            // Merge install orders (maintaining dependency order)
            for tool in result.install_order {
                if !combined_result.install_order.contains(&tool) {
                    combined_result.install_order.push(tool);
                }
            }

            // Merge other fields
            for tool in result.missing_tools {
                if !combined_result.missing_tools.contains(&tool) {
                    combined_result.missing_tools.push(tool);
                }
            }

            for tool in result.available_tools {
                if !combined_result.available_tools.contains(&tool) {
                    combined_result.available_tools.push(tool);
                }
            }

            combined_result
                .circular_dependencies
                .extend(result.circular_dependencies);
            combined_result
                .version_conflicts
                .extend(result.version_conflicts);
        }

        // Re-sort install order to ensure proper dependency ordering
        let final_order = {
            let mut graph = self.graph.write().await;
            graph.get_install_order(&combined_result.install_order)?
        };
        combined_result.install_order = final_order;

        Ok(combined_result)
    }

    /// Check if a tool is registered
    pub async fn is_tool_registered(&self, tool_name: &str) -> bool {
        let registry = self.tool_registry.read().await;
        registry.contains_key(tool_name)
    }

    /// Get tool specification
    pub async fn get_tool_spec(&self, tool_name: &str) -> Option<ToolSpec> {
        let registry = self.tool_registry.read().await;
        registry.get(tool_name).cloned()
    }

    /// Get all registered tools
    pub async fn get_all_tools(&self) -> Vec<String> {
        let registry = self.tool_registry.read().await;
        registry.keys().cloned().collect()
    }

    /// Clear resolution cache
    pub async fn clear_cache(&self) {
        let mut cache = self.resolution_cache.write().await;
        cache.clear();
    }

    /// Get dependency graph statistics
    pub async fn get_stats(&self) -> crate::graph::GraphStats {
        let graph = self.graph.read().await;
        graph.get_stats()
    }

    // Private helper methods

    async fn get_cached_resolution(&self, tool_name: &str) -> Option<CachedResolution> {
        let cache = self.resolution_cache.read().await;
        cache.get(tool_name).cloned()
    }

    async fn cache_resolution(&self, tool_name: &str, result: ResolutionResult) {
        let mut cache = self.resolution_cache.write().await;
        cache.insert(
            tool_name.to_string(),
            CachedResolution {
                result,
                cached_at: Instant::now(),
                ttl: self.options.cache_ttl,
            },
        );
    }

    async fn update_availability(&self) -> Result<()> {
        if let Some(checker) = &self.availability_checker {
            let tools = self.get_all_tools().await;
            let mut graph = self.graph.write().await;

            for tool_name in tools {
                let available = checker.is_available(&tool_name).await.unwrap_or(false);
                let version = if available {
                    checker.get_version(&tool_name).await.unwrap_or(None)
                } else {
                    None
                };

                graph.set_tool_available(&tool_name, available, version);
            }
        }
        Ok(())
    }

    async fn filter_resolution(&self, result: &mut ResolutionResult) {
        // Apply platform filter
        if let Some(platform) = &self.options.platform_filter {
            let registry = self.tool_registry.read().await;

            result.install_order.retain(|tool_name| {
                if let Some(tool_spec) = registry.get(tool_name) {
                    tool_spec
                        .dependencies
                        .iter()
                        .all(|dep| dep.applies_to_platform(platform))
                } else {
                    true
                }
            });
        }

        // Filter by dependency types
        if !self.options.include_optional || !self.options.include_dev {
            // TODO: Implement dependency type filtering
        }
    }
}

impl Default for ResolutionOptions {
    fn default() -> Self {
        Self {
            include_optional: false,
            include_dev: false,
            max_depth: 10,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            enable_parallel: true,
            platform_filter: None,
            allow_prerelease: false,
        }
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAvailabilityChecker {
        available_tools: HashMap<String, (bool, Option<String>)>,
    }

    impl MockAvailabilityChecker {
        fn new() -> Self {
            let mut available_tools = HashMap::new();
            available_tools.insert("node".to_string(), (true, Some("18.0.0".to_string())));
            available_tools.insert("python".to_string(), (true, Some("3.9.0".to_string())));

            Self { available_tools }
        }
    }

    #[async_trait::async_trait]
    impl AvailabilityChecker for MockAvailabilityChecker {
        async fn is_available(&self, tool_name: &str) -> Result<bool> {
            Ok(self
                .available_tools
                .get(tool_name)
                .map(|(available, _)| *available)
                .unwrap_or(false))
        }

        async fn get_version(&self, tool_name: &str) -> Result<Option<String>> {
            Ok(self
                .available_tools
                .get(tool_name)
                .and_then(|(_, version)| version.clone()))
        }

        async fn get_path(&self, _tool_name: &str) -> Result<Option<String>> {
            Ok(None)
        }
    }

    fn create_test_tool(name: &str, deps: Vec<&str>) -> ToolSpec {
        ToolSpec {
            name: name.to_string(),
            dependencies: deps
                .into_iter()
                .map(|dep| DependencySpec::required(dep, format!("{} requires {}", name, dep)))
                .collect(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_resolver_basic_functionality() {
        let resolver = DependencyResolver::new()
            .with_availability_checker(Arc::new(MockAvailabilityChecker::new()));

        // Register tools
        resolver
            .register_tool(create_test_tool("node", vec![]))
            .await
            .unwrap();
        resolver
            .register_tool(create_test_tool("yarn", vec!["node"]))
            .await
            .unwrap();

        // Resolve dependencies
        let result = resolver.resolve("yarn").await.unwrap();

        assert_eq!(result.install_order, vec!["node", "yarn"]);
        assert_eq!(result.available_tools, vec!["node"]);
        assert_eq!(result.missing_tools, vec!["yarn"]);
    }

    #[tokio::test]
    async fn test_resolver_multiple_tools() {
        let resolver = DependencyResolver::new()
            .with_availability_checker(Arc::new(MockAvailabilityChecker::new()));

        // Register tools
        resolver
            .register_tool(create_test_tool("node", vec![]))
            .await
            .unwrap();
        resolver
            .register_tool(create_test_tool("python", vec![]))
            .await
            .unwrap();
        resolver
            .register_tool(create_test_tool("yarn", vec!["node"]))
            .await
            .unwrap();
        resolver
            .register_tool(create_test_tool("pip", vec!["python"]))
            .await
            .unwrap();

        // Resolve multiple tools
        let result = resolver
            .resolve_multiple(&["yarn".to_string(), "pip".to_string()])
            .await
            .unwrap();

        // Should include all dependencies
        assert!(result.install_order.contains(&"node".to_string()));
        assert!(result.install_order.contains(&"python".to_string()));
        assert!(result.install_order.contains(&"yarn".to_string()));
        assert!(result.install_order.contains(&"pip".to_string()));
    }

    #[tokio::test]
    async fn test_resolver_caching() {
        let resolver = DependencyResolver::new()
            .with_availability_checker(Arc::new(MockAvailabilityChecker::new()));

        resolver
            .register_tool(create_test_tool("node", vec![]))
            .await
            .unwrap();
        resolver
            .register_tool(create_test_tool("yarn", vec!["node"]))
            .await
            .unwrap();

        // First resolution
        let start = Instant::now();
        let result1 = resolver.resolve("yarn").await.unwrap();
        let first_duration = start.elapsed();

        // Second resolution (should be cached)
        let start = Instant::now();
        let result2 = resolver.resolve("yarn").await.unwrap();
        let second_duration = start.elapsed();

        // Results should be identical
        assert_eq!(result1.install_order, result2.install_order);

        // Second call should be faster (cached)
        assert!(second_duration < first_duration);
    }

    #[tokio::test]
    async fn test_resolver_unregistered_tool() {
        let resolver = DependencyResolver::new();

        let result = resolver.resolve("nonexistent").await;
        assert!(result.is_err());

        if let Err(Error::ToolNotFound { tool }) = result {
            assert_eq!(tool, "nonexistent");
        } else {
            panic!("Expected ToolNotFound error");
        }
    }
}
