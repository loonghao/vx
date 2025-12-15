//! Provider registry
//!
//! The registry manages all registered providers and provides
//! lookup functionality.

use crate::provider::Provider;
use crate::runtime::Runtime;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for all providers
///
/// The registry provides a central place to register and look up
/// providers and their runtimes.
///
/// # Example
///
/// ```rust,no_run
/// use vx_runtime::ProviderRegistry;
///
/// let mut registry = ProviderRegistry::new();
/// // registry.register(Arc::new(NodeProvider::new()));
/// // registry.register(Arc::new(GoProvider::new()));
///
/// // Look up a runtime
/// if let Some(runtime) = registry.get_runtime("npm") {
///     println!("Found runtime: {}", runtime.name());
/// }
/// ```
pub struct ProviderRegistry {
    providers: RwLock<Vec<Arc<dyn Provider>>>,
    /// Cache: runtime name -> provider index
    runtime_cache: RwLock<HashMap<String, usize>>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(Vec::new()),
            runtime_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Register a provider
    pub fn register(&self, provider: Arc<dyn Provider>) {
        let mut providers = self.providers.write().unwrap();
        let index = providers.len();

        // Update cache for all runtimes in this provider
        {
            let mut cache = self.runtime_cache.write().unwrap();
            for runtime in provider.runtimes() {
                cache.insert(runtime.name().to_string(), index);
                for alias in runtime.aliases() {
                    cache.insert(alias.to_string(), index);
                }
            }
        }

        providers.push(provider);
    }

    /// Get a runtime by name or alias
    pub fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        // Check cache first
        let cache = self.runtime_cache.read().unwrap();
        if let Some(&index) = cache.get(name) {
            let providers = self.providers.read().unwrap();
            if let Some(provider) = providers.get(index) {
                return provider.get_runtime(name);
            }
        }

        // Fallback: search all providers
        let providers = self.providers.read().unwrap();
        for provider in providers.iter() {
            if let Some(runtime) = provider.get_runtime(name) {
                return Some(runtime);
            }
        }

        None
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn Provider>> {
        let providers = self.providers.read().unwrap();
        providers.iter().find(|p| p.name() == name).cloned()
    }

    /// Get all registered providers
    pub fn providers(&self) -> Vec<Arc<dyn Provider>> {
        self.providers.read().unwrap().clone()
    }

    /// Get all available runtime names
    pub fn runtime_names(&self) -> Vec<String> {
        let providers = self.providers.read().unwrap();
        let mut names = Vec::new();
        for provider in providers.iter() {
            for runtime in provider.runtimes() {
                names.push(runtime.name().to_string());
            }
        }
        names
    }

    /// Check if a runtime is supported
    pub fn supports(&self, name: &str) -> bool {
        self.get_runtime(name).is_some()
    }

    /// Clear all registered providers
    pub fn clear(&self) {
        self.providers.write().unwrap().clear();
        self.runtime_cache.write().unwrap().clear();
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
