//! Provider registry setup and management
//!
//! This module provides the registry setup for all providers.
//! It supports two registration modes:
//!
//! 1. **Static registration** (default): All providers are registered at compile time
//! 2. **Manifest-driven registration**: Providers are loaded from manifest files
//!
//! The manifest-driven approach is preferred as it provides a single source of truth
//! for provider metadata, but static registration is kept for backward compatibility.

use std::sync::Arc;
use vx_runtime::{create_runtime_context, ManifestRegistry, Provider, ProviderRegistry, Runtime, RuntimeContext};

/// Create and initialize the provider registry with all available providers
///
/// This uses static registration for maximum compatibility.
/// For manifest-driven registration, use `create_manifest_registry()`.
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

    // Register Vite provider (npm package)
    registry.register(vx_provider_vite::create_provider());

    // Register Rez provider (pip package)
    registry.register(vx_provider_rez::create_provider());

    // Register Deno provider
    registry.register(vx_provider_deno::create_provider());

    // Register Zig provider
    registry.register(vx_provider_zig::create_provider());

    // Register Java (Temurin JDK) provider
    registry.register(vx_provider_java::create_provider());

    // Register Terraform provider
    registry.register(vx_provider_terraform::create_provider());

    // Register kubectl provider
    registry.register(vx_provider_kubectl::create_provider());

    // Register Helm provider
    registry.register(vx_provider_helm::create_provider());

    // Register rcedit provider
    registry.register(vx_provider_rcedit::create_provider());

    // Register Git provider
    registry.register(vx_provider_git::create_provider());

    // Register Chocolatey provider
    registry.register(vx_provider_choco::create_provider());

    // Register Docker provider
    registry.register(vx_provider_docker::create_provider());

    // Register AWS CLI provider
    registry.register(vx_provider_awscli::create_provider());

    // Register Azure CLI provider
    registry.register(vx_provider_azcli::create_provider());

    // Register Google Cloud CLI provider
    registry.register(vx_provider_gcloud::create_provider());

    // Register Ninja provider
    registry.register(vx_provider_ninja::create_provider());

    // Register CMake provider
    registry.register(vx_provider_cmake::create_provider());

    // Register protoc provider
    registry.register(vx_provider_protoc::create_provider());

    // Register Task (go-task) provider
    registry.register(vx_provider_task::create_provider());

    // Register pre-commit provider
    registry.register(vx_provider_pre_commit::create_provider());

    // Register Ollama provider (AI tools)
    registry.register(vx_provider_ollama::create_provider());

    // Register Spack provider (HPC/Scientific computing)
    registry.register(vx_provider_spack::create_provider());

    // Register release-please provider (DevOps tools)
    registry.register(vx_provider_release_please::create_provider());

    // Register Python provider (using python-build-standalone)
    registry.register(vx_provider_python::create_provider());

    // Register MSVC Build Tools provider (Windows-only)
    registry.register(vx_provider_msvc::create_provider());

    registry
}

/// Create a manifest-driven registry with all builtin provider factories
///
/// This is the preferred approach as it uses manifest files as the source of truth.
/// The registry can optionally load additional manifests from a directory.
///
/// # Example
///
/// ```rust,ignore
/// // Create registry with builtin factories
/// let manifest_registry = create_manifest_registry();
///
/// // Build the provider registry
/// let provider_registry = manifest_registry.build_registry_from_factories();
/// ```
pub fn create_manifest_registry() -> ManifestRegistry {
    let mut registry = ManifestRegistry::new();

    // Register all builtin provider factories
    // This maps provider names to their create_provider() functions
    registry.register_factory("node", || vx_provider_node::create_provider());
    registry.register_factory("go", || vx_provider_go::create_provider());
    registry.register_factory("rust", || vx_provider_rust::create_provider());
    registry.register_factory("uv", || vx_provider_uv::create_provider());
    registry.register_factory("bun", || vx_provider_bun::create_provider());
    registry.register_factory("pnpm", || vx_provider_pnpm::create_provider());
    registry.register_factory("yarn", || vx_provider_yarn::create_provider());
    registry.register_factory("vscode", || vx_provider_vscode::create_provider());
    registry.register_factory("just", || vx_provider_just::create_provider());
    registry.register_factory("vite", || vx_provider_vite::create_provider());
    registry.register_factory("rez", || vx_provider_rez::create_provider());
    registry.register_factory("deno", || vx_provider_deno::create_provider());
    registry.register_factory("zig", || vx_provider_zig::create_provider());
    registry.register_factory("java", || vx_provider_java::create_provider());
    registry.register_factory("terraform", || vx_provider_terraform::create_provider());
    registry.register_factory("kubectl", || vx_provider_kubectl::create_provider());
    registry.register_factory("helm", || vx_provider_helm::create_provider());
    registry.register_factory("rcedit", || vx_provider_rcedit::create_provider());
    registry.register_factory("git", || vx_provider_git::create_provider());
    registry.register_factory("choco", || vx_provider_choco::create_provider());
    registry.register_factory("docker", || vx_provider_docker::create_provider());
    registry.register_factory("awscli", || vx_provider_awscli::create_provider());
    registry.register_factory("azcli", || vx_provider_azcli::create_provider());
    registry.register_factory("gcloud", || vx_provider_gcloud::create_provider());
    registry.register_factory("ninja", || vx_provider_ninja::create_provider());
    registry.register_factory("cmake", || vx_provider_cmake::create_provider());
    registry.register_factory("protoc", || vx_provider_protoc::create_provider());
    registry.register_factory("task", || vx_provider_task::create_provider());
    registry.register_factory("pre-commit", || vx_provider_pre_commit::create_provider());
    registry.register_factory("ollama", || vx_provider_ollama::create_provider());
    registry.register_factory("spack", || vx_provider_spack::create_provider());
    registry.register_factory("release-please", || vx_provider_release_please::create_provider());
    registry.register_factory("python", || vx_provider_python::create_provider());
    registry.register_factory("msvc", || vx_provider_msvc::create_provider());

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
