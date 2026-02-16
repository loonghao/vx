//! Provider builder
//!
//! Builds a `ProviderRegistry` from manifests + registered factories.
//! Returns a `BuildResult` with structured warnings and errors instead of
//! silently logging.

use crate::{Provider, ProviderRegistry};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, trace};
use vx_manifest::ProviderManifest;

/// Result of building a provider registry
pub struct BuildResult {
    /// The constructed provider registry
    pub registry: ProviderRegistry,
    /// Non-fatal warnings encountered during build
    pub warnings: Vec<BuildWarning>,
    /// Errors encountered during build (manifests without factories)
    pub errors: Vec<BuildError>,
}

impl std::fmt::Debug for BuildResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuildResult")
            .field("providers", &self.registry.providers().len())
            .field("warnings", &self.warnings)
            .field("errors", &self.errors)
            .finish()
    }
}

/// A non-fatal warning during provider build
#[derive(Debug, Clone)]
pub struct BuildWarning {
    /// Provider name
    pub provider: String,
    /// Warning message
    pub message: String,
}

impl std::fmt::Display for BuildWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.provider, self.message)
    }
}

/// Error category for provider build failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildErrorKind {
    /// Provider has a manifest but no Rust factory implementation yet
    NoFactory,
    /// Provider factory failed during construction
    FactoryFailed,
    /// Other error
    Other,
}

/// An error during provider build (manifest without matching factory)
#[derive(Debug, Clone)]
pub struct BuildError {
    /// Provider name
    pub provider: String,
    /// Optional runtime name (if error is runtime-specific)
    pub runtime: Option<String>,
    /// Error reason
    pub reason: String,
    /// Error category
    pub kind: BuildErrorKind,
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref runtime) = self.runtime {
            write!(f, "[{}/{}] {}", self.provider, runtime, self.reason)
        } else {
            write!(f, "[{}] {}", self.provider, self.reason)
        }
    }
}

impl BuildError {
    /// Check if this is a "no factory" error (manifest-only provider)
    pub fn is_no_factory(&self) -> bool {
        self.kind == BuildErrorKind::NoFactory
    }
}

/// Builder that constructs a `ProviderRegistry` from manifests + factories
///
/// The key improvement over the old `ManifestRegistry.build_registry()` is that
/// this returns a `BuildResult` with structured errors and warnings, rather than
/// silently logging warnings for missing factories.
pub struct ProviderBuilder {
    /// Provider factories by name
    factories: HashMap<String, Box<dyn Fn() -> Arc<dyn Provider> + Send + Sync>>,
}

impl Default for ProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderBuilder {
    /// Create a new empty builder
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a provider factory
    pub fn register_factory<F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> Arc<dyn Provider> + Send + Sync + 'static,
    {
        self.factories.insert(name.to_string(), Box::new(factory));
    }

    /// List all registered factory names
    pub fn factory_names(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    /// Build a `ProviderRegistry` from the given manifests
    ///
    /// Returns a `BuildResult` containing the registry along with any
    /// warnings or errors encountered during the build.
    pub fn build(&self, manifests: &[ProviderManifest]) -> BuildResult {
        let registry = ProviderRegistry::new();
        let warnings = Vec::new();
        let mut errors = Vec::new();

        for manifest in manifests {
            let name = &manifest.provider.name;

            match self.factories.get(name) {
                Some(factory) => {
                    let provider = factory();
                    registry.register(provider);
                    trace!("registered provider '{}'", name);
                }
                None => {
                    errors.push(BuildError {
                        provider: name.clone(),
                        runtime: None,
                        reason: "No Rust factory registered (manifest-only provider)".to_string(),
                        kind: BuildErrorKind::NoFactory,
                    });
                }
            }
        }

        info!(
            "built {} providers ({} errors, {} warnings)",
            registry.providers().len(),
            errors.len(),
            warnings.len(),
        );

        BuildResult {
            registry,
            warnings,
            errors,
        }
    }

    /// Build a `ProviderRegistry` using only registered factories (no manifest required)
    ///
    /// Useful for backward compatibility when manifests are not available.
    pub fn build_from_factories(&self) -> ProviderRegistry {
        let registry = ProviderRegistry::new();

        for (name, factory) in &self.factories {
            let provider = factory();
            registry.register(provider);
            trace!("registered provider '{}' from factory", name);
        }

        info!(
            "loaded {} providers from factories",
            registry.providers().len()
        );

        registry
    }

    /// Build a `ProviderRegistry` with lazy loading from the given manifests.
    ///
    /// Instead of immediately calling all factory functions, this method
    /// extracts runtime names and aliases from manifests and stores them
    /// alongside the factory closures. Providers are only instantiated
    /// when their runtimes are first accessed via `get_runtime()`.
    ///
    /// This significantly reduces startup time since only the provider
    /// needed for the current command is constructed.
    pub fn build_lazy(self, manifests: &[ProviderManifest]) -> BuildResult {
        let registry = ProviderRegistry::new();
        let warnings = Vec::new();
        let mut errors = Vec::new();
        let mut lazy_count = 0;

        // Move ownership of factories out of self
        let mut factories = self.factories;

        trace!("build_lazy: {} factories available", factories.len());

        for manifest in manifests {
            let name = &manifest.provider.name;

            trace!("build_lazy: processing manifest '{}' ({} runtimes)", name, manifest.runtimes.len());

            match factories.remove(name) {
                Some(factory) => {
                    // Collect all runtime names and aliases from the manifest
                    let mut runtime_names: Vec<String> = Vec::new();
                    for runtime in &manifest.runtimes {
                        runtime_names.push(runtime.name.clone());
                        for alias in &runtime.aliases {
                            runtime_names.push(alias.clone());
                        }
                    }

                    registry.register_lazy(name.clone(), runtime_names.clone(), factory);
                    lazy_count += 1;
                    tracing::debug!("registered lazy provider '{}' with runtimes: {:?}", name, runtime_names);
                }
                None => {
                    // Collect runtime names from manifest for better error message
                    let runtime_names: Vec<&str> = manifest
                        .runtimes
                        .iter()
                        .map(|r| r.name.as_str())
                        .collect();

                    errors.push(BuildError {
                        provider: name.clone(),
                        runtime: None,
                        reason: format!(
                            "No Rust factory registered (manifest-only provider, runtimes: [{}]). \
                             To enable this provider, add a Rust implementation and register it in registry.rs",
                            runtime_names.join(", ")
                        ),
                        kind: BuildErrorKind::NoFactory,
                    });
                }
            }
        }

        let no_factory_count = errors.iter().filter(|e| e.is_no_factory()).count();
        let real_error_count = errors.len() - no_factory_count;

        if no_factory_count > 0 {
            tracing::debug!(
                "{} provider(s) have manifests but no Rust factory (manifest-only)",
                no_factory_count
            );
        }

        info!(
            "registered {} lazy providers ({} errors, {} manifest-only, {} warnings)",
            lazy_count,
            real_error_count,
            no_factory_count,
            warnings.len(),
        );

        BuildResult {
            registry,
            warnings,
            errors,
        }
    }
}
