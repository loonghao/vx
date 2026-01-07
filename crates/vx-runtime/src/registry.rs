//! Provider registry
//!
//! The registry manages all registered providers and provides
//! lookup functionality.

use crate::plugin::ProviderLoader;
use crate::provider::Provider;
use crate::runtime::Runtime;
use crate::Platform;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

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
    /// Optional dynamic provider loader
    provider_loader: RwLock<Option<Arc<dyn ProviderLoader>>>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(Vec::new()),
            runtime_cache: RwLock::new(HashMap::new()),
            provider_loader: RwLock::new(None),
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

    /// Set a dynamic provider loader for loading providers on-demand
    pub fn set_provider_loader(&self, loader: Arc<dyn ProviderLoader>) {
        *self.provider_loader.write().unwrap() = Some(loader);
    }

    /// Get the provider loader if set
    pub fn get_provider_loader(&self) -> Option<Arc<dyn ProviderLoader>> {
        self.provider_loader.read().unwrap().clone()
    }

    /// Get all runtimes that support the current platform
    ///
    /// This filters out runtimes that are not available on the current OS/architecture.
    pub fn supported_runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        let current = Platform::current();
        self.supported_runtimes_for(&current)
    }

    /// Get all runtimes that support a specific platform
    pub fn supported_runtimes_for(&self, platform: &Platform) -> Vec<Arc<dyn Runtime>> {
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

    /// Get all runtimes grouped by platform support status
    ///
    /// Returns a tuple of (supported, unsupported) runtimes for the current platform.
    pub fn runtimes_by_platform_support(&self) -> (Vec<Arc<dyn Runtime>>, Vec<Arc<dyn Runtime>>) {
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
