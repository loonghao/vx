//! Provider registry
//!
//! The registry manages all registered providers and provides
//! lookup functionality.
//!
//! ## Lazy Loading
//!
//! The registry supports lazy provider loading: factory closures and manifest
//! metadata are stored at startup, but providers are only instantiated when
//! their runtimes are actually requested via `get_runtime()`. This avoids
//! constructing all ~45 providers on every `vx` invocation.
//!
//! Methods that need all providers (e.g., `providers()`, `runtime_names()`,
//! `supported_runtimes()`) will materialize all pending factories first.

use crate::Platform;
use crate::plugin::ProviderLoader;
use crate::provider::Provider;
use crate::runtime::Runtime;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;

/// Type alias for a provider factory closure
type ProviderFactory = Box<dyn Fn() -> Arc<dyn Provider> + Send + Sync>;

/// Registry for all providers
///
/// The registry provides a central place to register and look up
/// providers and their runtimes.
///
/// Supports two modes:
/// - **Eager**: providers are registered immediately via `register()`
/// - **Lazy**: factories are registered via `register_lazy()` and only
///   materialized on first access via `get_runtime()`
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
    /// Cache: runtime name -> provider index (for already-materialized providers)
    runtime_cache: RwLock<HashMap<String, usize>>,
    /// Optional dynamic provider loader
    provider_loader: RwLock<Option<Arc<dyn ProviderLoader>>>,
    /// Pending (not yet materialized) provider factories, keyed by provider name.
    /// Protected by Mutex because we need to take ownership of the factory to call it.
    pending_factories: Mutex<HashMap<String, ProviderFactory>>,
    /// Index: runtime name or alias -> provider name (for pending factories).
    /// Used to find which factory to materialize when `get_runtime()` is called.
    pending_index: RwLock<HashMap<String, String>>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(Vec::new()),
            runtime_cache: RwLock::new(HashMap::new()),
            provider_loader: RwLock::new(None),
            pending_factories: Mutex::new(HashMap::new()),
            pending_index: RwLock::new(HashMap::new()),
        }
    }

    /// Register a provider (eager — immediately materialized)
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

    /// Register a lazy provider factory.
    ///
    /// The factory will only be called when a runtime belonging to this provider
    /// is requested. `runtime_names` lists the runtime names and aliases that
    /// should trigger materialization of this factory.
    pub fn register_lazy(
        &self,
        provider_name: String,
        runtime_names: Vec<String>,
        factory: ProviderFactory,
    ) {
        // Build the pending index: runtime name/alias → provider name
        {
            let mut index = self.pending_index.write().unwrap();
            for name in runtime_names {
                // Detect alias conflicts: warn if a runtime name/alias is already
                // claimed by a different provider. The last writer wins, which can
                // cause non-deterministic behavior depending on manifest load order.
                if let Some(existing) = index.get(&name)
                    && existing != &provider_name
                {
                    tracing::warn!(
                        runtime = %name,
                        existing_provider = %existing,
                        new_provider = %provider_name,
                        "Runtime name/alias conflict: '{}' is claimed by both '{}' and '{}'. \
                         The last-registered provider wins, which may cause non-deterministic behavior.",
                        name, existing, provider_name
                    );
                }
                index.insert(name, provider_name.clone());
            }
        }

        // Store the factory
        {
            let mut factories = self.pending_factories.lock().unwrap();
            factories.insert(provider_name, factory);
        }
    }

    /// Materialize a single pending provider by its provider name.
    ///
    /// Returns `true` if the provider was materialized, `false` if it was
    /// already materialized or no factory was found.
    fn materialize_provider(&self, provider_name: &str) -> bool {
        // Take the factory out of pending (if present)
        let factory = {
            let mut factories = self.pending_factories.lock().unwrap();
            factories.remove(provider_name)
        };

        let Some(factory) = factory else {
            return false;
        };

        // Call the factory and register eagerly
        let provider = factory();
        tracing::trace!("lazy-loaded provider '{}'", provider_name);

        // Remove all pending index entries for this provider
        {
            let mut index = self.pending_index.write().unwrap();
            index.retain(|_, v| v != provider_name);
        }

        // Register the provider (this updates runtime_cache)
        self.register(provider);
        true
    }

    /// Materialize all pending providers.
    ///
    /// Called by methods that need the full set of providers (e.g., `providers()`,
    /// `runtime_names()`, `supported_runtimes()`).
    fn materialize_all(&self) {
        // Drain all pending factories
        let factories: Vec<(String, ProviderFactory)> = {
            let mut pending = self.pending_factories.lock().unwrap();
            pending.drain().collect()
        };

        if factories.is_empty() {
            return;
        }

        // Clear the pending index
        {
            let mut index = self.pending_index.write().unwrap();
            index.clear();
        }

        // Materialize each factory
        for (name, factory) in factories {
            let provider = factory();
            tracing::trace!("lazy-loaded provider '{}' (bulk)", name);
            self.register(provider);
        }
    }

    /// Check if there are any pending (not yet materialized) factories
    pub fn has_pending(&self) -> bool {
        let factories = self.pending_factories.lock().unwrap();
        !factories.is_empty()
    }

    /// Get a runtime by name or alias
    pub fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        // Fast path: check runtime_cache for already-materialized providers
        {
            let cache = self.runtime_cache.read().unwrap();
            if let Some(&index) = cache.get(name) {
                let providers = self.providers.read().unwrap();
                if let Some(provider) = providers.get(index) {
                    return provider.get_runtime(name);
                }
            }
        }

        // Check if there's a pending factory for this runtime name
        let provider_name = {
            let index = self.pending_index.read().unwrap();
            index.get(name).cloned()
        };

        if let Some(provider_name) = provider_name {
            // Materialize the provider on demand
            self.materialize_provider(&provider_name);

            // Now look up in the freshly populated cache
            let cache = self.runtime_cache.read().unwrap();
            if let Some(&index) = cache.get(name) {
                let providers = self.providers.read().unwrap();
                if let Some(provider) = providers.get(index) {
                    return provider.get_runtime(name);
                }
            }
        }

        // Fallback: search all materialized providers
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
        // Try materialized providers first
        {
            let providers = self.providers.read().unwrap();
            if let Some(p) = providers.iter().find(|p| p.name() == name) {
                return Some(p.clone());
            }
        }

        // Try to materialize from pending
        if self.materialize_provider(name) {
            let providers = self.providers.read().unwrap();
            return providers.iter().find(|p| p.name() == name).cloned();
        }

        None
    }

    /// Get all registered providers (materializes all pending factories)
    pub fn providers(&self) -> Vec<Arc<dyn Provider>> {
        self.materialize_all();
        self.providers.read().unwrap().clone()
    }

    /// Get all available runtime names (materializes all pending factories)
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
        // Check materialized cache first
        {
            let cache = self.runtime_cache.read().unwrap();
            if cache.contains_key(name) {
                return true;
            }
        }

        // Check pending index
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

    /// Set a dynamic provider loader for loading providers on-demand
    pub fn set_provider_loader(&self, loader: Arc<dyn ProviderLoader>) {
        *self.provider_loader.write().unwrap() = Some(loader);
    }

    /// Get the provider loader if set
    pub fn get_provider_loader(&self) -> Option<Arc<dyn ProviderLoader>> {
        self.provider_loader.read().unwrap().clone()
    }

    /// Get all runtimes that support the current platform (materializes all pending)
    ///
    /// This filters out runtimes that are not available on the current OS/architecture.
    pub fn supported_runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        let current = Platform::current();
        self.supported_runtimes_for(&current)
    }

    /// Get all runtimes that support a specific platform (materializes all pending)
    pub fn supported_runtimes_for(&self, platform: &Platform) -> Vec<Arc<dyn Runtime>> {
        self.materialize_all();
        let providers = self.providers.read().unwrap();
        let mut runtimes = Vec::new();

        for provider in providers.iter() {
            // Check if provider supports the platform
            if !provider.is_platform_supported(platform) {
                continue;
            }

            // Add runtimes that support the platform
            for runtime in provider.runtimes() {
                if runtime.is_platform_supported(platform) {
                    runtimes.push(runtime);
                }
            }
        }

        runtimes
    }

    /// Get a runtime by name, checking platform compatibility
    ///
    /// Returns an error if the runtime exists but doesn't support the current platform.
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

    /// Get all runtimes grouped by platform support status (materializes all pending)
    ///
    /// Returns a tuple of (supported, unsupported) runtimes for the current platform.
    #[allow(clippy::type_complexity)]
    pub fn runtimes_by_platform_support(&self) -> (Vec<Arc<dyn Runtime>>, Vec<Arc<dyn Runtime>>) {
        self.materialize_all();
        let current = Platform::current();
        let providers = self.providers.read().unwrap();
        let mut supported = Vec::new();
        let mut unsupported = Vec::new();

        for provider in providers.iter() {
            for runtime in provider.runtimes() {
                if runtime.is_platform_supported(&current) {
                    supported.push(runtime);
                } else {
                    unsupported.push(runtime);
                }
            }
        }

        (supported, unsupported)
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

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
