//! Provider registry setup and management
//!
//! This module provides the registry setup for all providers.

use std::sync::Arc;
use vx_runtime::{create_runtime_context, Provider, ProviderRegistry, Runtime, RuntimeContext};

/// Create and initialize the provider registry with all available providers
pub fn create_registry() -> ProviderRegistry {
    let registry = ProviderRegistry::new();

    // Register Node.js provider
    registry.register(vx_provider_node::create_provider());

    // Register Go provider
    registry.register(vx_provider_go::create_provider());

    // Register Rust provider
    registry.register(vx_provider_rust::create_provider());

    // Register UV provider
    registry.register(vx_provider_uv::create_provider());

    // Register Bun provider
    registry.register(vx_provider_bun::create_provider());

    // Register Pnpm provider
    registry.register(vx_provider_pnpm::create_provider());

    // Register Yarn provider
    registry.register(vx_provider_yarn::create_provider());

    // Register VSCode provider
    registry.register(vx_provider_vscode::create_provider());

    // Register Just provider
    registry.register(vx_provider_just::create_provider());

    registry
}

/// Create a runtime context for operations
pub fn create_context() -> anyhow::Result<RuntimeContext> {
    create_runtime_context()
}

/// Extension trait for ProviderRegistry to provide convenience methods
pub trait ProviderRegistryExt {
    /// Get a runtime by name
    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>>;

    /// Check if a runtime is supported
    fn supports_runtime(&self, name: &str) -> bool;

    /// List all runtime names
    fn list_runtimes(&self) -> Vec<String>;

    /// Get all providers
    fn get_providers(&self) -> Vec<Arc<dyn Provider>>;
}

impl ProviderRegistryExt for ProviderRegistry {
    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        ProviderRegistry::get_runtime(self, name)
    }

    fn supports_runtime(&self, name: &str) -> bool {
        self.supports(name)
    }

    fn list_runtimes(&self) -> Vec<String> {
        self.runtime_names()
    }

    fn get_providers(&self) -> Vec<Arc<dyn Provider>> {
        self.providers()
    }
}
