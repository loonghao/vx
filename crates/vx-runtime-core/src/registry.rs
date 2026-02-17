//! Provider registry
//!
//! The registry manages all registered providers and provides
//! lookup functionality.
//!
//! ## Lazy Loading
//!
//! The registry supports lazy provider loading: factory closures are stored
//! at startup, but providers are only instantiated when their runtimes are
//! actually requested via `get_runtime()`.

use crate::platform::Platform;
use crate::provider::Provider;
use crate::runtime::Runtime;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;

/// Type alias for a provider factory closure
type ProviderFactory = Box<dyn Fn() -> Arc<dyn Provider> + Send + Sync>;

/// Registry for all providers
pub struct ProviderRegistry {
    providers: RwLock<Vec<Arc<dyn Provider>>>,
    /// Cache: runtime name -> provider index
    runtime_cache: RwLock<HashMap<String, usize>>,
    /// Pending (not yet materialized) provider factories
    pending_factories: Mutex<HashMap<String, ProviderFactory>>,
    /// Index: runtime name or alias -> provider name
    pending_index: RwLock<HashMap<String, String>>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(Vec::new()),
            runtime_cache: RwLock::new(HashMap::new()),
            pending_factories: Mutex::new(HashMap::new()),
            pending_index: RwLock::new(HashMap::new()),
        }
    }

    /// Register a provider (eager — immediately materialized)
    pub fn register(&self, provider: Arc<dyn Provider>) {
        let mut providers = self.providers.write().unwrap();
        let index = providers.len();

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

    /// Register a lazy provider factory
    pub fn register_lazy(
        &self,
        provider_name: String,
        runtime_names: Vec<String>,
        factory: ProviderFactory,
    ) {
        tracing::trace!(
            "register_lazy: provider='{}', runtimes={:?}",
            provider_name,
            runtime_names
        );

        {
            let mut index = self.pending_index.write().unwrap();
            for name in runtime_names {
                if let Some(existing) = index.get(&name)
                    && existing != &provider_name
                {
                    tracing::warn!(
                        runtime = %name,
                        existing_provider = %existing,
                        new_provider = %provider_name,
                        "Runtime name/alias conflict"
                    );
                }
                index.insert(name, provider_name.clone());
            }
        }

        {
            let mut factories = self.pending_factories.lock().unwrap();
            factories.insert(provider_name, factory);
        }
    }

    /// Materialize a single pending provider by its provider name
    fn materialize_provider(&self, provider_name: &str) -> bool {
        let factory = {
            let mut factories = self.pending_factories.lock().unwrap();
            factories.remove(provider_name)
        };

        let Some(factory) = factory else {
            return false;
        };

        let provider = factory();
        tracing::trace!("lazy-loaded provider '{}'", provider_name);

        {
            let mut index = self.pending_index.write().unwrap();
            index.retain(|_, v| v != provider_name);
        }

        self.register(provider);
        true
    }

    /// Materialize all pending providers
    fn materialize_all(&self) {
        let factories: Vec<(String, ProviderFactory)> = {
            let mut pending = self.pending_factories.lock().unwrap();
            pending.drain().collect()
        };

        if factories.is_empty() {
            return;
        }

        {
            let mut index = self.pending_index.write().unwrap();
            index.clear();
        }

        for (name, factory) in factories {
            let provider = factory();
            tracing::trace!("lazy-loaded provider '{}' (bulk)", name);
            self.register(provider);
        }
    }

    /// Check if there are any pending factories
    pub fn has_pending(&self) -> bool {
        let factories = self.pending_factories.lock().unwrap();
        !factories.is_empty()
    }

    /// Get the count of pending factories
    pub fn pending_factories_count(&self) -> usize {
        let factories = self.pending_factories.lock().unwrap();
        factories.len()
    }

    /// Get a runtime by name or alias
    pub fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        // Fast path: check runtime_cache
        {
            let cache = self.runtime_cache.read().unwrap();
            if let Some(&index) = cache.get(name) {
                let providers = self.providers.read().unwrap();
                if let Some(provider) = providers.get(index) {
                    return provider.get_runtime(name);
                }
            }
        }

        // Check pending index
        let provider_name = {
            let index = self.pending_index.read().unwrap();
            index.get(name).cloned()
        };

        if let Some(provider_name) = provider_name {
            self.materialize_provider(&provider_name);

            let cache = self.runtime_cache.read().unwrap();
            if let Some(&index) = cache.get(name) {
                let providers = self.providers.read().unwrap();
                if let Some(provider) = providers.get(index) {
                    return provider.get_runtime(name);
                }
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
        {
            let providers = self.providers.read().unwrap();
            if let Some(p) = providers.iter().find(|p| p.name() == name) {
                return Some(p.clone());
            }
        }

        if self.materialize_provider(name) {
            let providers = self.providers.read().unwrap();
            return providers.iter().find(|p| p.name() == name).cloned();
        }

        None
    }

    /// Get all registered providers
    pub fn providers(&self) -> Vec<Arc<dyn Provider>> {
        self.materialize_all();
        self.providers.read().unwrap().clone()
    }

    /// Get all available runtime names
    pub fn runtime_names(&self) -> Vec<String> {
        self.materialize_all();
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
        {
            let cache = self.runtime_cache.read().unwrap();
            if cache.contains_key(name) {
                return true;
            }
        }

        {
            let index = self.pending_index.read().unwrap();
            if index.contains_key(name) {
                return true;
            }
        }

        false
    }

    /// Clear all registered providers and pending factories
    pub fn clear(&self) {
        self.providers.write().unwrap().clear();
        self.runtime_cache.write().unwrap().clear();
        self.pending_factories.lock().unwrap().clear();
        self.pending_index.write().unwrap().clear();
    }

    /// Get all runtimes that support the current platform
    pub fn supported_runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        let current = Platform::current();
        self.supported_runtimes_for(&current)
    }

    /// Get all runtimes that support a specific platform
    pub fn supported_runtimes_for(&self, platform: &Platform) -> Vec<Arc<dyn Runtime>> {
        self.materialize_all();
        let providers = self.providers.read().unwrap();
        let mut runtimes = Vec::new();

        for provider in providers.iter() {
            if !provider.is_platform_supported(platform) {
                continue;
            }

            for runtime in provider.runtimes() {
                if runtime.is_platform_supported(platform) {
                    runtimes.push(runtime);
                }
            }
        }

        runtimes
    }

    /// Get a runtime by name, checking platform compatibility
    pub fn get_runtime_checked(&self, name: &str) -> Result<Arc<dyn Runtime>, PlatformError> {
        let current = Platform::current();
        self.get_runtime_checked_for(name, &current)
    }

    /// Get a runtime by name, checking compatibility with a specific platform
    pub fn get_runtime_checked_for(
        &self,
        name: &str,
        platform: &Platform,
    ) -> Result<Arc<dyn Runtime>, PlatformError> {
        if let Some(runtime) = self.get_runtime(name) {
            if runtime.is_platform_supported(platform) {
                Ok(runtime)
            } else {
                Err(PlatformError::UnsupportedPlatform {
                    runtime: name.to_string(),
                    current_os: platform.os.as_str().to_string(),
                    current_arch: platform.arch.as_str().to_string(),
                    supported: runtime
                        .supported_platforms()
                        .iter()
                        .map(|p| format!("{}-{}", p.os.as_str(), p.arch.as_str()))
                        .collect(),
                })
            }
        } else {
            Err(PlatformError::NotFound(name.to_string()))
        }
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Platform-related errors
#[derive(Debug, Error)]
pub enum PlatformError {
    /// Runtime not found
    #[error("Runtime '{0}' not found")]
    NotFound(String),

    /// Runtime not supported on current platform
    #[error("Runtime '{runtime}' is not available on {current_os}-{current_arch}. Supported platforms: {}", supported.join(", "))]
    UnsupportedPlatform {
        runtime: String,
        current_os: String,
        current_arch: String,
        supported: Vec<String>,
    },
}
